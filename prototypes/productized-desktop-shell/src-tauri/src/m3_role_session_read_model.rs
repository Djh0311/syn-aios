//! M3 RoleSession read-model boundary.
//!
//! This module is intentionally a *read* boundary.  It does not manufacture a
//! RoleSession binding from a renderer hint, a legacy thread, a frontend cache,
//! or an Agent/Supervisor profile. M3C07 keeps its isolated acceptance
//! constructor, while M4C02 adds a separate ordinary-product Secretary host
//! whose PersonalScope binding is constructed only by the backend. Existing
//! Agent/Jiaoban commands stay project-scoped and fail closed when only the
//! Secretary runtime is installed.

use crate::m3_role_session::{
    ConversationContextRef, OpaqueRef, ProviderHandleRef, RoleSession, RoleSessionId,
    RoleSessionState, ServerResolvedBinding, Sha256Digest,
};
use crate::m3_role_session_repository::{
    M3ConversationContextReadDto, M3ConversationContextReadState, M3ReadPermissionDisposition,
    M3RoleSessionDirectoryCursor, M3RoleSessionDirectoryQuery, M3RoleSessionReadSnapshot,
    M3RoleSessionSnapshotQuery, M3RoleSessionSqliteRepository, M3SessionBindingReadState,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// A deliberately stable, public failure code.  Do not replace this with an
/// inferred profile, a legacy thread lookup, or a cache fallback.
pub(crate) const M3_BINDING_UNAVAILABLE: &str = "M3_BINDING_UNAVAILABLE";

const MAX_PROJECT_LOCATOR_BYTES: usize = 1024;
const MAX_NONCE_BYTES: usize = 160;
const MAX_SELECTOR_BYTES: usize = 256;
const MAX_CONTINUATION_MESSAGE_BYTES: usize = 32 * 1024;
const MAX_SELECTOR_RECORDS: usize = 2048;
static NEXT_SELECTOR_RUNTIME: AtomicU64 = AtomicU64::new(0);

/// The renderer never submits this enum.  Fixed Tauri commands select it at
/// the host boundary, so an Agent renderer cannot become a Jiaoban renderer by
/// changing a JSON field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum M3RoleSessionReadHost {
    Agent,
    Jiaoban,
}

/// The ordinary-product Secretary is a distinct server-only host. Keeping it
/// out of `M3RoleSessionReadHost` prevents a personal scope from being routed
/// through the project-locator commands or from changing M3C07 host semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct M3SecretaryReadHost;

impl M3SecretaryReadHost {
    pub(crate) fn server_fixed() -> Self {
        Self
    }

    fn as_str(self) -> &'static str {
        "SECRETARY"
    }
}

/// Renderer input for a host-fixed directory endpoint.  `project_locator` is
/// a non-authoritative routing hint: the injected server binding must have an
/// exact canonical match before the repository is queried.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M3RoleSessionDirectoryRequest {
    pub(crate) project_locator: String,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<u32>,
    pub(crate) request_nonce: String,
}

/// Renderer input for a host-fixed detail endpoint.  It deliberately has no
/// `role_session_id`, role, scope, current object, channel, actor, permission,
/// provider handle, thread, or profile field.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M3RoleSessionDetailRequest {
    pub(crate) project_locator: String,
    pub(crate) selection: String,
    pub(crate) request_nonce: String,
}

/// The only existing-session send shape accepted by the M3 surface.  The
/// message is transient and is not persisted or logged by this module.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M3RoleSessionContinuationStartRequest {
    pub(crate) project_locator: String,
    pub(crate) continuation_selector: String,
    pub(crate) request_nonce: String,
    pub(crate) user_text: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M3RoleSessionPermissionReadState {
    Current,
    RevalidationRequired,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M3RoleSessionContextState {
    Available,
    Missing,
    NeedsReprojection,
    SessionFailClosed,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M3RoleSessionContinuationState {
    Available,
    Disabled,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3RoleSessionDisplayLabelsDto {
    pub(crate) role_label: String,
    pub(crate) project_label: String,
    pub(crate) object_label: String,
    pub(crate) channel_label: String,
    pub(crate) permission_label: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3RoleSessionSourceLinkDto {
    pub(crate) source_ref: Option<String>,
    pub(crate) label: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3RoleSessionContextDto {
    pub(crate) state: M3RoleSessionContextState,
    pub(crate) retrieval_status: Option<String>,
    pub(crate) context_sources: Vec<String>,
    pub(crate) knowledge_refs: Vec<String>,
    pub(crate) gaps: Vec<String>,
    pub(crate) source_links: Vec<M3RoleSessionSourceLinkDto>,
    pub(crate) request_more_material_available: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3RoleSessionContinuationDto {
    pub(crate) state: M3RoleSessionContinuationState,
    pub(crate) selector: Option<String>,
    pub(crate) reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3RoleSessionDirectoryEntryDto {
    pub(crate) selection: String,
    pub(crate) role_session_id: String,
    pub(crate) session_revision: u64,
    pub(crate) labels: M3RoleSessionDisplayLabelsDto,
    pub(crate) session_state: String,
    pub(crate) permission_state: M3RoleSessionPermissionReadState,
    pub(crate) resolution_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3RoleSessionDirectoryDto {
    pub(crate) request_nonce: String,
    pub(crate) projection_revision: String,
    pub(crate) entries: Vec<M3RoleSessionDirectoryEntryDto>,
    pub(crate) next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3RoleSessionDetailDto {
    pub(crate) request_nonce: String,
    pub(crate) selection: String,
    pub(crate) role_session_id: String,
    pub(crate) session_revision: u64,
    pub(crate) projection_revision: String,
    pub(crate) labels: M3RoleSessionDisplayLabelsDto,
    pub(crate) session_state: String,
    pub(crate) permission_state: M3RoleSessionPermissionReadState,
    pub(crate) resolution_reason: Option<String>,
    pub(crate) context: M3RoleSessionContextDto,
    pub(crate) continuation: M3RoleSessionContinuationDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct M3SecretaryRoleSessionStatusDto {
    pub(crate) host: String,
    pub(crate) role_session_id: String,
    pub(crate) session_revision: u64,
    pub(crate) session_state: String,
    pub(crate) actor_id: String,
    pub(crate) role_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) current_object_ref: String,
    pub(crate) execution_channel: String,
    pub(crate) permission_snapshot_ref: String,
    pub(crate) owner_fingerprint: String,
}

/// This crate-private value crosses only from the read-model authorization
/// gate to the guarded M3 transport adapter.  It is never serialized to a
/// renderer and intentionally keeps the provider handle out of all DTOs.
#[derive(Clone, Debug)]
pub(crate) struct M3RoleSessionContinuationGuard {
    pub(crate) role_session_id: RoleSessionId,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) expected_session_revision: u64,
    pub(crate) binding_revision: u64,
    pub(crate) provider_handle_ref: ProviderHandleRef,
    pub(crate) context_ref: ConversationContextRef,
    pub(crate) context_metadata_hash: Sha256Digest,
}

#[derive(Clone)]
pub(crate) struct M3RoleSessionReadRuntimeSlot {
    runtime: Option<M3RoleSessionReadRuntime>,
}

impl Default for M3RoleSessionReadRuntimeSlot {
    fn default() -> Self {
        Self { runtime: None }
    }
}

impl M3RoleSessionReadRuntimeSlot {
    /// Creates the one production-capable read runtime.  Its deliberately
    /// specific name prevents profile/cache/thread callers from treating this
    /// as a generic authority injector; only `m3_acceptance` can mint the
    /// required bindings after the M3C07 gate has passed.
    pub(crate) fn from_m3c07_isolated_acceptance(bindings: Vec<M3C07IsolatedReadBinding>) -> Self {
        Self {
            runtime: Some(M3RoleSessionReadRuntime::m3c07_isolated(bindings)),
        }
    }

    /// Installs the ordinary-product Secretary runtime after M4 has resolved
    /// and bootstrapped one exact PersonalScope RoleSession. This constructor
    /// cannot install Agent/Jiaoban project bindings and does not accept a
    /// project locator, cwd, renderer profile, or acceptance permit.
    pub(crate) fn from_ordinary_product_secretary(
        entry: M3OrdinarySecretaryReadBinding,
    ) -> Result<Self, String> {
        Ok(Self {
            runtime: Some(M3RoleSessionReadRuntime::ordinary_product_secretary(entry)?),
        })
    }

    pub(crate) fn secretary_status(&self) -> Result<M3SecretaryRoleSessionStatusDto, String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())?
            .secretary_status()
    }

    pub(crate) fn directory(
        &self,
        host: M3RoleSessionReadHost,
        request: &M3RoleSessionDirectoryRequest,
    ) -> Result<M3RoleSessionDirectoryDto, String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())?
            .directory(host, request)
    }

    pub(crate) fn detail(
        &self,
        host: M3RoleSessionReadHost,
        request: &M3RoleSessionDetailRequest,
    ) -> Result<M3RoleSessionDetailDto, String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())?
            .detail(host, request)
    }

    pub(crate) fn authorize_continuation(
        &self,
        host: M3RoleSessionReadHost,
        request: &M3RoleSessionContinuationStartRequest,
    ) -> Result<M3RoleSessionContinuationGuard, String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())?
            .authorize_continuation(host, request)
    }
}

/// A binding minted exclusively by the M3C07 acceptance runtime after it has
/// verified both the R4 isolated profile and the explicit M3C07 mode gate.
///
/// This is intentionally not a renderer-facing type and it is not a general
/// runtime setter.  Normal production startup has no constructor for it, so
/// the regular M3 commands remain `M3_BINDING_UNAVAILABLE`.
#[derive(Clone)]
pub(crate) struct M3C07IsolatedReadBinding {
    pub(crate) host: M3RoleSessionReadHost,
    pub(crate) project_locator: String,
    pub(crate) repository: M3RoleSessionSqliteRepository,
    pub(crate) binding: ServerResolvedBinding,
}

#[derive(Clone)]
pub(crate) struct M3OrdinarySecretaryReadBinding {
    pub(crate) host: M3SecretaryReadHost,
    pub(crate) repository: M3RoleSessionSqliteRepository,
    pub(crate) binding: ServerResolvedBinding,
    pub(crate) role_session_id: RoleSessionId,
}

#[derive(Clone)]
struct M3RoleSessionReadRuntime {
    repository_by_host_project:
        Arc<BTreeMap<(M3RoleSessionReadHost, String), M3ReadRepositoryBinding>>,
    secretary: Option<M3SecretaryReadRepositoryBinding>,
    selectors: Arc<Mutex<M3SelectorStore>>,
    selector_counter: Arc<AtomicU64>,
    selector_namespace: String,
}

#[derive(Clone)]
struct M3ReadRepositoryBinding {
    repository: M3RoleSessionSqliteRepository,
    binding: ServerResolvedBinding,
}

#[derive(Clone)]
struct M3SecretaryReadRepositoryBinding {
    host: M3SecretaryReadHost,
    repository: M3RoleSessionSqliteRepository,
    binding: ServerResolvedBinding,
    role_session_id: RoleSessionId,
}

impl M3RoleSessionReadRuntime {
    fn m3c07_isolated(bindings: Vec<M3C07IsolatedReadBinding>) -> Self {
        let mut by_host_project = BTreeMap::new();
        for entry in bindings {
            // Test fixture setup is still forced through the same canonical
            // locator and binding validation as a command request.
            if validate_project_locator(&entry.project_locator).is_err()
                || entry.binding.verify_owner_fingerprint().is_err()
            {
                continue;
            }
            by_host_project.insert(
                (entry.host, entry.project_locator),
                M3ReadRepositoryBinding {
                    repository: entry.repository,
                    binding: entry.binding,
                },
            );
        }
        Self {
            repository_by_host_project: Arc::new(by_host_project),
            secretary: None,
            selectors: Arc::new(Mutex::new(M3SelectorStore::default())),
            selector_counter: Arc::new(AtomicU64::new(0)),
            selector_namespace: selector_runtime_namespace(),
        }
    }

    fn ordinary_product_secretary(entry: M3OrdinarySecretaryReadBinding) -> Result<Self, String> {
        let M3OrdinarySecretaryReadBinding {
            host,
            repository,
            binding,
            role_session_id,
        } = entry;
        binding
            .verify_owner_fingerprint()
            .map_err(|_| M3_BINDING_UNAVAILABLE.to_string())?;
        let snapshot = repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
            })
            .map_err(read_repository_error)?
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())?;
        if snapshot.session.status != RoleSessionState::Active
            || !snapshot.session.matches_binding_identity(&binding)
            || snapshot.session.permission_snapshot_ref != binding.permission_snapshot_ref
        {
            return Err(M3_BINDING_UNAVAILABLE.to_string());
        }
        Ok(Self {
            repository_by_host_project: Arc::new(BTreeMap::new()),
            secretary: Some(M3SecretaryReadRepositoryBinding {
                host,
                repository,
                binding,
                role_session_id,
            }),
            selectors: Arc::new(Mutex::new(M3SelectorStore::default())),
            selector_counter: Arc::new(AtomicU64::new(0)),
            selector_namespace: selector_runtime_namespace(),
        })
    }

    #[cfg(test)]
    fn isolated_fixture(bindings: Vec<M3C07IsolatedReadBinding>) -> Self {
        Self::m3c07_isolated(bindings)
    }

    fn directory(
        &self,
        host: M3RoleSessionReadHost,
        request: &M3RoleSessionDirectoryRequest,
    ) -> Result<M3RoleSessionDirectoryDto, String> {
        validate_project_locator(&request.project_locator)?;
        validate_request_nonce(&request.request_nonce)?;
        let read_binding = self.resolve_binding(host, &request.project_locator)?;
        let after = match request.cursor.as_deref() {
            Some(selector) => {
                match self.lookup_selector(host, &request.project_locator, selector)? {
                    M3SelectorKind::Cursor(cursor) => Some(cursor),
                    _ => return Err("m3_role_session_cursor_kind_invalid".to_string()),
                }
            }
            None => None,
        };
        let limit = request.limit.unwrap_or(50);
        let page = read_binding
            .repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding: read_binding.binding.clone(),
                after,
                limit,
            })
            .map_err(read_repository_error)?;
        let mut entries = Vec::with_capacity(page.entries.len());
        for entry in page.entries {
            let selection = self.mint_selector(
                host,
                &request.project_locator,
                M3SelectorKind::Selection {
                    role_session_id: entry.session.role_session_id.clone(),
                    session_revision: entry.session.revision,
                },
            );
            entries.push(M3RoleSessionDirectoryEntryDto {
                selection,
                role_session_id: entry.session.role_session_id.as_str().to_string(),
                session_revision: entry.session.revision,
                labels: labels_for_session(&entry.session),
                session_state: entry.session.status.as_str().to_string(),
                permission_state: permission_state(&entry.permission),
                resolution_reason: entry
                    .session
                    .resolution_reason
                    .map(|reason| reason.as_str().to_string()),
            });
        }
        let next_cursor = page.next_cursor.map(|cursor| {
            self.mint_selector(
                host,
                &request.project_locator,
                M3SelectorKind::Cursor(cursor),
            )
        });
        Ok(M3RoleSessionDirectoryDto {
            request_nonce: request.request_nonce.clone(),
            projection_revision: projection_revision_for_directory(&entries),
            entries,
            next_cursor,
        })
    }

    fn detail(
        &self,
        host: M3RoleSessionReadHost,
        request: &M3RoleSessionDetailRequest,
    ) -> Result<M3RoleSessionDetailDto, String> {
        validate_project_locator(&request.project_locator)?;
        validate_request_nonce(&request.request_nonce)?;
        let read_binding = self.resolve_binding(host, &request.project_locator)?;
        let selection = self.lookup_selector(host, &request.project_locator, &request.selection)?;
        let (role_session_id, selected_revision) = match selection {
            M3SelectorKind::Selection {
                role_session_id,
                session_revision,
            } => (role_session_id, session_revision),
            _ => return Err("m3_role_session_selection_kind_invalid".to_string()),
        };
        let snapshot = read_binding
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: role_session_id.clone(),
                binding: read_binding.binding.clone(),
            })
            .map_err(read_repository_error)?
            .ok_or_else(|| "m3_role_session_selection_not_found".to_string())?;
        if snapshot.session.revision != selected_revision {
            return Err("m3_role_session_selection_stale".to_string());
        }
        self.detail_from_snapshot(
            host,
            &request.project_locator,
            &request.request_nonce,
            &request.selection,
            snapshot,
        )
    }

    fn authorize_continuation(
        &self,
        host: M3RoleSessionReadHost,
        request: &M3RoleSessionContinuationStartRequest,
    ) -> Result<M3RoleSessionContinuationGuard, String> {
        validate_project_locator(&request.project_locator)?;
        validate_request_nonce(&request.request_nonce)?;
        validate_continuation_message(&request.user_text)?;
        let read_binding = self.resolve_binding(host, &request.project_locator)?;
        let selector = self.lookup_selector(
            host,
            &request.project_locator,
            &request.continuation_selector,
        )?;
        let (role_session_id, session_revision, projection_revision) = match selector {
            M3SelectorKind::Continuation {
                role_session_id,
                session_revision,
                projection_revision,
            } => (role_session_id, session_revision, projection_revision),
            _ => return Err("m3_role_session_continuation_kind_invalid".to_string()),
        };
        let snapshot = read_binding
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: role_session_id.clone(),
                binding: read_binding.binding.clone(),
            })
            .map_err(read_repository_error)?
            .ok_or_else(|| "m3_role_session_continuation_not_found".to_string())?;
        if snapshot.session.revision != session_revision
            || projection_revision_for_snapshot(&snapshot) != projection_revision
        {
            return Err("m3_role_session_continuation_stale".to_string());
        }
        continuation_guard_for_snapshot(&snapshot, read_binding.binding.clone())
    }

    fn secretary_status(&self) -> Result<M3SecretaryRoleSessionStatusDto, String> {
        let read_binding = self
            .secretary
            .as_ref()
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())?;
        let snapshot = read_binding
            .repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: read_binding.role_session_id.clone(),
                binding: read_binding.binding.clone(),
            })
            .map_err(read_repository_error)?
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())?;
        if snapshot.session.status != RoleSessionState::Active
            || !snapshot
                .session
                .matches_binding_identity(&read_binding.binding)
            || snapshot.session.permission_snapshot_ref
                != read_binding.binding.permission_snapshot_ref
        {
            return Err(M3_BINDING_UNAVAILABLE.to_string());
        }
        Ok(M3SecretaryRoleSessionStatusDto {
            host: read_binding.host.as_str().to_string(),
            role_session_id: snapshot.session.role_session_id.as_str().to_string(),
            session_revision: snapshot.session.revision,
            session_state: snapshot.session.status.as_str().to_string(),
            actor_id: snapshot.session.actor_id.as_str().to_string(),
            role_ref: snapshot.session.role_ref.as_str().to_string(),
            scope_ref: snapshot.session.scope_ref.as_str().to_string(),
            current_object_ref: snapshot.session.current_object_ref.as_str().to_string(),
            execution_channel: snapshot.session.execution_channel.as_str().to_string(),
            permission_snapshot_ref: snapshot
                .session
                .permission_snapshot_ref
                .as_str()
                .to_string(),
            owner_fingerprint: snapshot.session.owner_fingerprint.as_str().to_string(),
        })
    }

    fn detail_from_snapshot(
        &self,
        host: M3RoleSessionReadHost,
        project_locator: &str,
        request_nonce: &str,
        selection: &str,
        snapshot: M3RoleSessionReadSnapshot,
    ) -> Result<M3RoleSessionDetailDto, String> {
        let projection_revision = projection_revision_for_snapshot(&snapshot);
        let continuation = match continuation_guard_for_snapshot(
            &snapshot,
            self.resolve_binding(host, project_locator)?.binding,
        ) {
            Ok(_) => M3RoleSessionContinuationDto {
                state: M3RoleSessionContinuationState::Available,
                selector: Some(self.mint_selector(
                    host,
                    project_locator,
                    M3SelectorKind::Continuation {
                        role_session_id: snapshot.session.role_session_id.clone(),
                        session_revision: snapshot.session.revision,
                        projection_revision: projection_revision.clone(),
                    },
                )),
                reason: None,
            },
            Err(reason) => M3RoleSessionContinuationDto {
                state: M3RoleSessionContinuationState::Disabled,
                selector: None,
                reason: Some(reason),
            },
        };
        Ok(M3RoleSessionDetailDto {
            request_nonce: request_nonce.to_string(),
            selection: selection.to_string(),
            role_session_id: snapshot.session.role_session_id.as_str().to_string(),
            session_revision: snapshot.session.revision,
            projection_revision,
            labels: labels_for_session(&snapshot.session),
            session_state: snapshot.session.status.as_str().to_string(),
            permission_state: permission_state(&snapshot.permission),
            resolution_reason: snapshot
                .session
                .resolution_reason
                .map(|reason| reason.as_str().to_string()),
            context: context_dto(&snapshot.current_context),
            continuation,
        })
    }

    fn resolve_binding(
        &self,
        host: M3RoleSessionReadHost,
        project_locator: &str,
    ) -> Result<M3ReadRepositoryBinding, String> {
        self.repository_by_host_project
            .get(&(host, project_locator.to_string()))
            .cloned()
            .ok_or_else(|| M3_BINDING_UNAVAILABLE.to_string())
    }

    fn mint_selector(
        &self,
        host: M3RoleSessionReadHost,
        project_locator: &str,
        kind: M3SelectorKind,
    ) -> String {
        let sequence = self.selector_counter.fetch_add(1, Ordering::Relaxed) + 1;
        let selector = format!("m3rs:{}:{:x}", self.selector_namespace, sequence);
        let mut store = self
            .selectors
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        store.insert(
            selector.clone(),
            M3SelectorRecord {
                host,
                project_locator: project_locator.to_string(),
                kind,
            },
        );
        selector
    }

    fn lookup_selector(
        &self,
        host: M3RoleSessionReadHost,
        project_locator: &str,
        selector: &str,
    ) -> Result<M3SelectorKind, String> {
        validate_selector(selector)?;
        let store = self
            .selectors
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let record = store
            .records
            .get(selector)
            .ok_or_else(|| "m3_role_session_selector_unknown".to_string())?;
        if record.host != host || record.project_locator != project_locator {
            return Err("m3_role_session_selector_binding_mismatch".to_string());
        }
        Ok(record.kind.clone())
    }
}

fn selector_runtime_namespace() -> String {
    let runtime_sequence = NEXT_SELECTOR_RUNTIME.fetch_add(1, Ordering::Relaxed) + 1;
    let seed = format!(
        "m3-role-session-selector-runtime:{}:{runtime_sequence}",
        std::process::id(),
    );
    Sha256Digest::of_bytes(seed.as_bytes())
        .as_str()
        .chars()
        .take(24)
        .collect()
}

#[derive(Clone)]
enum M3SelectorKind {
    Selection {
        role_session_id: RoleSessionId,
        session_revision: u64,
    },
    Cursor(M3RoleSessionDirectoryCursor),
    Continuation {
        role_session_id: RoleSessionId,
        session_revision: u64,
        projection_revision: String,
    },
}

#[derive(Clone)]
struct M3SelectorRecord {
    host: M3RoleSessionReadHost,
    project_locator: String,
    kind: M3SelectorKind,
}

#[derive(Default)]
struct M3SelectorStore {
    records: BTreeMap<String, M3SelectorRecord>,
    order: VecDeque<String>,
}

impl M3SelectorStore {
    fn insert(&mut self, selector: String, record: M3SelectorRecord) {
        while self.records.len() >= MAX_SELECTOR_RECORDS {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            self.records.remove(&oldest);
        }
        self.order.push_back(selector.clone());
        self.records.insert(selector, record);
    }
}

fn labels_for_session(session: &RoleSession) -> M3RoleSessionDisplayLabelsDto {
    M3RoleSessionDisplayLabelsDto {
        role_label: session.role_ref.as_str().to_string(),
        project_label: session.scope_ref.as_str().to_string(),
        object_label: session.current_object_ref.as_str().to_string(),
        channel_label: session.execution_channel.as_str().to_string(),
        permission_label: session.permission_snapshot_ref.as_str().to_string(),
    }
}

fn permission_state(permission: &M3ReadPermissionDisposition) -> M3RoleSessionPermissionReadState {
    match permission {
        M3ReadPermissionDisposition::Current => M3RoleSessionPermissionReadState::Current,
        M3ReadPermissionDisposition::RevalidationRequired { .. } => {
            M3RoleSessionPermissionReadState::RevalidationRequired
        }
    }
}

fn context_dto(state: &M3ConversationContextReadState) -> M3RoleSessionContextDto {
    match state {
        M3ConversationContextReadState::Available(context) => context_dto_from_available(context),
        M3ConversationContextReadState::Missing => M3RoleSessionContextDto {
            state: M3RoleSessionContextState::Missing,
            retrieval_status: None,
            context_sources: Vec::new(),
            knowledge_refs: Vec::new(),
            gaps: vec!["CONTEXT_MISSING".to_string()],
            source_links: Vec::new(),
            request_more_material_available: false,
        },
        M3ConversationContextReadState::NeedsReprojection => M3RoleSessionContextDto {
            state: M3RoleSessionContextState::NeedsReprojection,
            retrieval_status: None,
            context_sources: Vec::new(),
            knowledge_refs: Vec::new(),
            gaps: vec!["CONTEXT_REPROJECTION_REQUIRED".to_string()],
            source_links: Vec::new(),
            request_more_material_available: false,
        },
        M3ConversationContextReadState::SessionFailClosed => M3RoleSessionContextDto {
            state: M3RoleSessionContextState::SessionFailClosed,
            retrieval_status: None,
            context_sources: Vec::new(),
            knowledge_refs: Vec::new(),
            gaps: vec!["SESSION_FAIL_CLOSED".to_string()],
            source_links: Vec::new(),
            request_more_material_available: false,
        },
    }
}

fn context_dto_from_available(context: &M3ConversationContextReadDto) -> M3RoleSessionContextDto {
    let source_refs = refs_to_strings(&context.context.source_refs);
    let mut knowledge_refs = refs_to_strings(&context.context.included_material_refs);
    knowledge_refs.extend(refs_to_strings(&context.context.included_skill_refs));
    knowledge_refs.sort();
    knowledge_refs.dedup();
    let mut gaps = refs_to_strings(&context.context.known_gaps);
    gaps.extend(refs_to_strings(
        &context.context.known_conflicts_or_uncertainties,
    ));
    gaps.extend(
        context
            .context
            .excluded_material_refs_with_reason
            .iter()
            .map(|excluded| {
                format!(
                    "{}:{}",
                    excluded.reason.as_str(),
                    excluded.material_ref.as_str()
                )
            }),
    );
    gaps.sort();
    gaps.dedup();
    let source_links = context
        .context
        .source_link_labels
        .iter()
        .enumerate()
        .map(|(index, label)| M3RoleSessionSourceLinkDto {
            source_ref: source_refs.get(index).cloned(),
            label: label.as_str().to_string(),
        })
        .collect();
    M3RoleSessionContextDto {
        state: M3RoleSessionContextState::Available,
        retrieval_status: Some(context.context.retrieval_status.as_str().to_string()),
        context_sources: source_refs,
        knowledge_refs,
        gaps,
        source_links,
        request_more_material_available: context.context.request_more_material_ref.is_some(),
    }
}

fn continuation_guard_for_snapshot(
    snapshot: &M3RoleSessionReadSnapshot,
    binding: ServerResolvedBinding,
) -> Result<M3RoleSessionContinuationGuard, String> {
    if snapshot.session.status.as_str() == "QUARANTINED" {
        return Err("SESSION_QUARANTINED".to_string());
    }
    if snapshot.session.status.as_str() == "CLOSED" {
        return Err("SESSION_CLOSED".to_string());
    }
    if snapshot.session.status.as_str() != "ACTIVE" {
        return Err("SESSION_NOT_ACTIVE".to_string());
    }
    if !matches!(snapshot.permission, M3ReadPermissionDisposition::Current) {
        return Err("PERMISSION_REVALIDATION_REQUIRED".to_string());
    }
    let (binding_revision, provider_handle_ref) = match &snapshot.current_binding {
        M3SessionBindingReadState::Verified {
            binding_revision,
            provider_handle_ref,
        } => (*binding_revision, provider_handle_ref.clone()),
        M3SessionBindingReadState::UnboundSessionStart => {
            return Err("SESSION_BINDING_UNAVAILABLE".to_string())
        }
        M3SessionBindingReadState::RevalidationRequired => {
            return Err("SESSION_REVALIDATION_REQUIRED".to_string())
        }
        M3SessionBindingReadState::SessionFailClosed => {
            return Err("SESSION_FAIL_CLOSED".to_string())
        }
    };
    let context = match &snapshot.current_context {
        M3ConversationContextReadState::Available(context) => context,
        M3ConversationContextReadState::Missing => return Err("CONTEXT_MISSING".to_string()),
        M3ConversationContextReadState::NeedsReprojection => {
            return Err("CONTEXT_REPROJECTION_REQUIRED".to_string())
        }
        M3ConversationContextReadState::SessionFailClosed => {
            return Err("SESSION_FAIL_CLOSED".to_string())
        }
    };
    if context.context.retrieval_status.as_str() != "COMPLETE" {
        return Err("CONTEXT_RETRIEVAL_INCOMPLETE".to_string());
    }
    if !context.context.known_gaps.is_empty()
        || !context.context.known_conflicts_or_uncertainties.is_empty()
        || !context
            .context
            .excluded_material_refs_with_reason
            .is_empty()
    {
        return Err("CONTEXT_GAPS_PRESENT".to_string());
    }
    if !snapshot.session.matches_binding_identity(&binding)
        || snapshot.session.permission_snapshot_ref != binding.permission_snapshot_ref
        || context.context.role_session_id != snapshot.session.role_session_id
        || context.context.scope_ref != binding.scope_ref
        || context.context.current_object_ref != binding.current_object_ref
        || context.permission_snapshot_ref != binding.permission_snapshot_ref
        || context.binding_revision != binding_revision
    {
        return Err("SERVER_BINDING_REVALIDATION_REQUIRED".to_string());
    }
    Ok(M3RoleSessionContinuationGuard {
        role_session_id: snapshot.session.role_session_id.clone(),
        binding,
        expected_session_revision: snapshot.session.revision,
        binding_revision,
        provider_handle_ref,
        context_ref: context.context.context_ref.clone(),
        context_metadata_hash: context.context_metadata_hash.clone(),
    })
}

fn projection_revision_for_snapshot(snapshot: &M3RoleSessionReadSnapshot) -> String {
    match &snapshot.current_context {
        M3ConversationContextReadState::Available(context) => format!(
            "{}:{}:{}",
            snapshot.session.revision, context.binding_revision, context.context.projection_version
        ),
        _ => format!("{}:none", snapshot.session.revision),
    }
}

fn projection_revision_for_directory(entries: &[M3RoleSessionDirectoryEntryDto]) -> String {
    let mut stable = String::new();
    for entry in entries {
        stable.push_str(&entry.role_session_id);
        stable.push(':');
        stable.push_str(&entry.session_revision.to_string());
        stable.push(';');
    }
    format!(
        "directory:{}",
        Sha256Digest::of_bytes(stable.as_bytes()).as_str()
    )
}

fn refs_to_strings(refs: &[OpaqueRef]) -> Vec<String> {
    let mut unique = BTreeSet::new();
    for reference in refs {
        unique.insert(reference.as_str().to_string());
    }
    unique.into_iter().collect()
}

fn read_repository_error(
    error: crate::m3_role_session_repository::M3RoleSessionRepositoryError,
) -> String {
    // Repository failures are already scrubbed stable codes.  Do not use
    // Display formatting with user request material at this boundary.
    error.code
}

fn validate_project_locator(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.as_bytes().len() > MAX_PROJECT_LOCATOR_BYTES
        || value.chars().any(char::is_control)
    {
        return Err("m3_role_session_project_locator_invalid".to_string());
    }
    Ok(())
}

fn validate_request_nonce(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.as_bytes().len() > MAX_NONCE_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-'))
    {
        return Err("m3_role_session_request_nonce_invalid".to_string());
    }
    Ok(())
}

fn validate_selector(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.as_bytes().len() > MAX_SELECTOR_BYTES
        || !value.starts_with("m3rs:")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-'))
    {
        return Err("m3_role_session_selector_invalid".to_string());
    }
    Ok(())
}

fn validate_continuation_message(value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.as_bytes().len() > MAX_CONTINUATION_MESSAGE_BYTES
        || value.chars().any(char::is_control)
    {
        return Err("m3_role_session_continuation_message_invalid".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session::{
        ConversationContext, CorrelationId, ExcludedMaterialReason, ExcludedMaterialReference,
        RequestIdempotencyKey, RetrievalStatus, RoleSessionState,
    };
    use crate::m3_role_session_repository::{
        CreateRoleSessionCommand, M3CommandMetadata, M3RoleSessionSnapshotQuery,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct ReadModelFixture {
        path: PathBuf,
        repository: M3RoleSessionSqliteRepository,
        binding: ServerResolvedBinding,
        role_session_id: RoleSessionId,
    }

    impl ReadModelFixture {
        fn empty(tag: &str) -> Self {
            let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "syn-m3c06-read-model-{tag}-{}-{sequence}.sqlite",
                std::process::id(),
            ));
            Self {
                repository: M3RoleSessionSqliteRepository::open_rehearsal(&path)
                    .expect("open M3C06 scratch repository"),
                path,
                binding: binding_for(tag, "v1"),
                role_session_id: role_session_id_for(tag),
            }
        }

        fn active(tag: &str) -> Self {
            let fixture = Self::empty(tag);
            fixture
                .repository
                .create_role_session(&CreateRoleSessionCommand {
                    role_session_id: fixture.role_session_id.clone(),
                    binding: fixture.binding.clone(),
                    metadata: metadata(&format!("{tag}:create")),
                })
                .expect("create fixture RoleSession");
            fixture
        }

        fn runtime(
            &self,
            host: M3RoleSessionReadHost,
            project_locator: &str,
            binding: ServerResolvedBinding,
        ) -> M3RoleSessionReadRuntime {
            M3RoleSessionReadRuntime::isolated_fixture(vec![M3C07IsolatedReadBinding {
                host,
                project_locator: project_locator.to_string(),
                repository: self.repository.clone(),
                binding,
            }])
        }

        fn snapshot(&self) -> M3RoleSessionReadSnapshot {
            self.repository
                .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                    role_session_id: self.role_session_id.clone(),
                    binding: self.binding.clone(),
                })
                .expect("load fixture RoleSession")
                .expect("fixture RoleSession is present")
        }
    }

    impl Drop for ReadModelFixture {
        fn drop(&mut self) {
            for suffix in ["", "-wal", "-shm"] {
                let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", self.path.display())));
            }
        }
    }

    fn sealed(namespace: &str, value: impl AsRef<str>) -> String {
        format!(
            "{namespace}:sha256:{}",
            Sha256Digest::of_bytes(value.as_ref().as_bytes()).as_str()
        )
    }

    fn opaque(namespace: &str, value: impl AsRef<str>) -> OpaqueRef {
        OpaqueRef::try_from_canonical(sealed(namespace, value)).expect("sealed fixture ref")
    }

    fn role_session_id_for(tag: &str) -> RoleSessionId {
        RoleSessionId::try_from_canonical(sealed("session", tag)).expect("sealed session id")
    }

    fn provider_handle_ref_for(tag: &str) -> ProviderHandleRef {
        ProviderHandleRef::try_from_canonical(sealed("handle", tag))
            .expect("sealed provider handle ref")
    }

    fn context_ref_for(tag: &str) -> ConversationContextRef {
        ConversationContextRef::try_from_canonical(sealed("context", tag))
            .expect("sealed context ref")
    }

    fn binding_for(tag: &str, permission_version: &str) -> ServerResolvedBinding {
        ServerResolvedBinding::from_server_canonical(
            sealed("actor", tag),
            sealed("role", "worker"),
            sealed("scope", tag),
            sealed("object", tag),
            sealed("channel", "agent"),
            sealed("permission", format!("{tag}:{permission_version}")),
        )
        .expect("fixture server binding")
    }

    fn binding_with_permission(
        binding: &ServerResolvedBinding,
        tag: &str,
        permission_version: &str,
    ) -> ServerResolvedBinding {
        ServerResolvedBinding::from_server_canonical(
            binding.actor_id.as_str().to_string(),
            binding.role_ref.as_str().to_string(),
            binding.scope_ref.as_str().to_string(),
            binding.current_object_ref.as_str().to_string(),
            binding.execution_channel.as_str().to_string(),
            sealed("permission", format!("{tag}:{permission_version}")),
        )
        .expect("same-identity permission drift binding")
    }

    fn metadata(tag: &str) -> M3CommandMetadata {
        M3CommandMetadata {
            receipt_id: opaque("receipt", tag),
            event_id: opaque("event", tag),
            audit_id: opaque("audit", tag),
            correlation_id: CorrelationId::try_from_canonical(sealed("correlation", tag))
                .expect("sealed correlation"),
            request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed(
                "request", tag,
            ))
            .expect("sealed request key"),
            occurred_at: "2026-08-09T00:00:00Z".to_string(),
        }
    }

    fn directory_request(project_locator: &str, nonce: &str) -> M3RoleSessionDirectoryRequest {
        M3RoleSessionDirectoryRequest {
            project_locator: project_locator.to_string(),
            cursor: None,
            limit: Some(20),
            request_nonce: nonce.to_string(),
        }
    }

    fn available_context(
        session: &RoleSession,
        binding: &ServerResolvedBinding,
        tag: &str,
        with_gap: bool,
    ) -> M3ConversationContextReadDto {
        let context = ConversationContext {
            context_ref: context_ref_for(tag),
            role_session_id: session.role_session_id.clone(),
            objective_ref: opaque("objective", tag),
            scope_ref: binding.scope_ref.clone(),
            current_object_ref: binding.current_object_ref.clone(),
            source_refs: vec![opaque("source", tag)],
            included_material_refs: vec![opaque("material", tag)],
            included_skill_refs: vec![opaque("skill", tag)],
            source_watermark: opaque("watermark", tag),
            freshness_or_staleness_marker: opaque("freshness", tag),
            known_gaps: with_gap.then(|| opaque("gap", tag)).into_iter().collect(),
            known_conflicts_or_uncertainties: Vec::new(),
            excluded_material_refs_with_reason: with_gap
                .then(|| ExcludedMaterialReference {
                    material_ref: opaque("excluded", tag),
                    reason: ExcludedMaterialReason::PermissionDenied,
                })
                .into_iter()
                .collect(),
            retrieval_status: RetrievalStatus::Complete,
            request_more_material_ref: with_gap.then(|| opaque("request-more", tag)),
            scrubbed_summary_ref: Some(opaque("summary", tag)),
            source_link_labels: vec![opaque("source-label", tag)],
            projection_version: format!("projection:{tag}:v1"),
        };
        let context_metadata_hash =
            Sha256Digest::of_bytes(&serde_json::to_vec(&context).expect("serialize context"));
        M3ConversationContextReadDto {
            context,
            permission_snapshot_ref: binding.permission_snapshot_ref.clone(),
            binding_revision: 1,
            context_metadata_hash,
            updated_at: "2026-08-09T00:00:00Z".to_string(),
        }
    }

    fn available_snapshot(fixture: &ReadModelFixture, tag: &str) -> M3RoleSessionReadSnapshot {
        let mut snapshot = fixture.snapshot();
        snapshot.current_binding = M3SessionBindingReadState::Verified {
            binding_revision: 1,
            provider_handle_ref: provider_handle_ref_for(tag),
        };
        snapshot.current_context = M3ConversationContextReadState::Available(available_context(
            &snapshot.session,
            &fixture.binding,
            tag,
            false,
        ));
        snapshot
    }

    #[test]
    fn m4c02_secretary_runtime_is_server_only_and_keeps_project_hosts_closed() {
        let fixture = ReadModelFixture::active("m4c02-secretary-runtime");
        let slot = M3RoleSessionReadRuntimeSlot::from_ordinary_product_secretary(
            M3OrdinarySecretaryReadBinding {
                host: M3SecretaryReadHost::server_fixed(),
                repository: fixture.repository.clone(),
                binding: fixture.binding.clone(),
                role_session_id: fixture.role_session_id.clone(),
            },
        )
        .expect("install exact ordinary-product Secretary runtime");

        let status = slot
            .secretary_status()
            .expect("reload Secretary status from repository");
        assert_eq!(status.host, "SECRETARY");
        assert_eq!(status.role_session_id, fixture.role_session_id.as_str());
        assert_eq!(status.session_state, "ACTIVE");
        assert_eq!(status.scope_ref, fixture.binding.scope_ref.as_str());
        assert_eq!(
            status.owner_fingerprint,
            fixture.binding.owner_fingerprint.as_str()
        );

        assert_eq!(
            slot.directory(
                M3RoleSessionReadHost::Agent,
                &directory_request("/not-a-personal-scope", "secretary-project-probe"),
            )
            .expect_err("Secretary runtime must not become a project binding"),
            M3_BINDING_UNAVAILABLE,
        );
    }

    #[test]
    fn m4c02_secretary_runtime_rejects_missing_or_wrong_scope_binding() {
        assert_eq!(
            M3RoleSessionReadRuntimeSlot::default()
                .secretary_status()
                .expect_err("default AppState has no ordinary Secretary runtime"),
            M3_BINDING_UNAVAILABLE,
        );

        let fixture = ReadModelFixture::active("m4c02-secretary-wrong-scope");
        let wrong_scope = binding_for("m4c02-secretary-foreign-scope", "v1");
        let error = match M3RoleSessionReadRuntimeSlot::from_ordinary_product_secretary(
            M3OrdinarySecretaryReadBinding {
                host: M3SecretaryReadHost::server_fixed(),
                repository: fixture.repository.clone(),
                binding: wrong_scope,
                role_session_id: fixture.role_session_id.clone(),
            },
        ) {
            Ok(_) => panic!("cross-scope binding must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error, "m3_read_server_binding_mismatch");
    }

    #[test]
    fn m3c06_directory_reloads_from_backend_and_rejects_cross_project_or_stale_selection() {
        let fixture = ReadModelFixture::active("directory-reload");
        let foreign_binding = binding_for("directory-foreign", "v1");
        fixture
            .repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id_for("directory-foreign"),
                binding: foreign_binding,
                metadata: metadata("directory-foreign:create"),
            })
            .expect("create foreign fixture RoleSession");
        let project = "/m3c06/fixture/project";
        let first_runtime = fixture.runtime(
            M3RoleSessionReadHost::Agent,
            project,
            fixture.binding.clone(),
        );
        let first_page = first_runtime
            .directory(
                M3RoleSessionReadHost::Agent,
                &directory_request(project, "reload-1"),
            )
            .expect("read exact binding directory");
        assert_eq!(
            first_page.entries.len(),
            1,
            "foreign role session must stay hidden"
        );
        assert_eq!(
            first_page.entries[0].role_session_id,
            fixture.role_session_id.as_str()
        );
        let stale_selection = first_page.entries[0].selection.clone();

        let reloaded_runtime = fixture.runtime(
            M3RoleSessionReadHost::Agent,
            project,
            fixture.binding.clone(),
        );
        let reloaded_page = reloaded_runtime
            .directory(
                M3RoleSessionReadHost::Agent,
                &directory_request(project, "reload-2"),
            )
            .expect("reload must rebuild directory from repository evidence");
        let reloaded_selection = reloaded_page.entries[0].selection.clone();
        assert_eq!(
            reloaded_page.entries[0].role_session_id,
            fixture.role_session_id.as_str()
        );
        assert_ne!(
            stale_selection, reloaded_selection,
            "selectors are runtime-opaque"
        );
        assert_eq!(
            reloaded_runtime
                .detail(
                    M3RoleSessionReadHost::Agent,
                    &M3RoleSessionDetailRequest {
                        project_locator: project.to_string(),
                        selection: stale_selection,
                        request_nonce: "reload-stale-detail".to_string(),
                    },
                )
                .expect_err("a prior-runtime selection must not recover authority"),
            "m3_role_session_selector_unknown",
        );
        let detail = reloaded_runtime
            .detail(
                M3RoleSessionReadHost::Agent,
                &M3RoleSessionDetailRequest {
                    project_locator: project.to_string(),
                    selection: reloaded_selection,
                    request_nonce: "reload-current-detail".to_string(),
                },
            )
            .expect("fresh opaque selection resolves from repository");
        assert_eq!(detail.context.state, M3RoleSessionContextState::Missing);
        assert_eq!(
            detail.continuation.state,
            M3RoleSessionContinuationState::Disabled
        );
        assert_eq!(
            reloaded_runtime
                .directory(
                    M3RoleSessionReadHost::Agent,
                    &directory_request("/m3c06/fixture/other-project", "cross-project"),
                )
                .expect_err("a project locator cannot select another binding"),
            M3_BINDING_UNAVAILABLE,
        );
    }

    #[test]
    fn m3c06_empty_invalid_and_permission_drift_read_states_fail_closed() {
        let empty = ReadModelFixture::empty("empty-directory");
        let project = "/m3c06/fixture/empty";
        let empty_runtime = empty.runtime(
            M3RoleSessionReadHost::Jiaoban,
            project,
            empty.binding.clone(),
        );
        assert!(empty_runtime
            .directory(
                M3RoleSessionReadHost::Jiaoban,
                &directory_request(project, "empty-directory"),
            )
            .expect("empty backend directory is a valid state")
            .entries
            .is_empty());
        assert_eq!(
            empty_runtime
                .directory(
                    M3RoleSessionReadHost::Jiaoban,
                    &M3RoleSessionDirectoryRequest {
                        project_locator: project.to_string(),
                        cursor: None,
                        limit: Some(20),
                        request_nonce: "bad nonce with spaces".to_string(),
                    },
                )
                .expect_err("invalid renderer nonce is an error, not a fallback"),
            "m3_role_session_request_nonce_invalid",
        );

        let fixture = ReadModelFixture::active("permission-drift");
        let drift_binding = binding_with_permission(&fixture.binding, "permission-drift", "v2");
        let runtime = fixture.runtime(
            M3RoleSessionReadHost::Jiaoban,
            "/m3c06/fixture/drift",
            drift_binding,
        );
        let page = runtime
            .directory(
                M3RoleSessionReadHost::Jiaoban,
                &directory_request("/m3c06/fixture/drift", "permission-drift-page"),
            )
            .expect("same owner binding remains visible for revalidation state");
        assert_eq!(
            page.entries[0].permission_state,
            M3RoleSessionPermissionReadState::RevalidationRequired,
        );
        let detail = runtime
            .detail(
                M3RoleSessionReadHost::Jiaoban,
                &M3RoleSessionDetailRequest {
                    project_locator: "/m3c06/fixture/drift".to_string(),
                    selection: page.entries[0].selection.clone(),
                    request_nonce: "permission-drift-detail".to_string(),
                },
            )
            .expect("permission drift remains a redacted read state");
        assert_eq!(
            detail.context.state,
            M3RoleSessionContextState::NeedsReprojection
        );
        assert_eq!(
            detail.continuation.state,
            M3RoleSessionContinuationState::Disabled
        );
        assert_eq!(
            detail.continuation.reason.as_deref(),
            Some("PERMISSION_REVALIDATION_REQUIRED"),
        );
    }

    #[test]
    fn m3c06_context_gaps_quarantine_and_stale_continuation_never_reach_adapter() {
        let fixture = ReadModelFixture::active("continuation-gates");
        let project = "/m3c06/fixture/continuation";
        let runtime = fixture.runtime(
            M3RoleSessionReadHost::Agent,
            project,
            fixture.binding.clone(),
        );
        let snapshot = available_snapshot(&fixture, "continuation-gates");
        let detail = runtime
            .detail_from_snapshot(
                M3RoleSessionReadHost::Agent,
                project,
                "available-detail",
                "m3rs:fixture-selection",
                snapshot.clone(),
            )
            .expect("fixture projection is available");
        assert_eq!(detail.context.state, M3RoleSessionContextState::Available);
        assert_eq!(detail.context.context_sources.len(), 1);
        assert_eq!(detail.context.knowledge_refs.len(), 2);
        assert_eq!(detail.context.source_links.len(), 1);
        assert_eq!(
            detail.continuation.state,
            M3RoleSessionContinuationState::Available
        );
        let rendered = serde_json::to_string(&detail).expect("serialize redacted detail");
        for forbidden in [
            "owner_fingerprint",
            "provider_handle",
            "raw_transcript",
            "thread_id",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "DTO must not expose {forbidden}"
            );
        }
        let stale_selector = detail
            .continuation
            .selector
            .clone()
            .expect("available projection has opaque continuation selector");
        assert_eq!(
            runtime
                .authorize_continuation(
                    M3RoleSessionReadHost::Agent,
                    &M3RoleSessionContinuationStartRequest {
                        project_locator: project.to_string(),
                        continuation_selector: stale_selector,
                        request_nonce: "stale-continuation".to_string(),
                        user_text: "fixture message".to_string(),
                    },
                )
                .expect_err("repository reprojection changes invalidate selector"),
            "m3_role_session_continuation_stale",
        );

        let mut gap_snapshot = snapshot.clone();
        gap_snapshot.current_context =
            M3ConversationContextReadState::Available(available_context(
                &gap_snapshot.session,
                &fixture.binding,
                "continuation-gates-gap",
                true,
            ));
        let gap_detail = runtime
            .detail_from_snapshot(
                M3RoleSessionReadHost::Agent,
                project,
                "gap-detail",
                "m3rs:gap-selection",
                gap_snapshot,
            )
            .expect("gap projection remains readable");
        assert!(gap_detail
            .context
            .gaps
            .iter()
            .any(|gap| gap.contains("PERMISSION_DENIED")));
        assert!(gap_detail.context.request_more_material_available);
        assert_eq!(
            gap_detail.continuation.reason.as_deref(),
            Some("CONTEXT_GAPS_PRESENT"),
        );

        let mut quarantined_snapshot = snapshot;
        quarantined_snapshot.session.status = RoleSessionState::Quarantined;
        let quarantined_detail = runtime
            .detail_from_snapshot(
                M3RoleSessionReadHost::Agent,
                project,
                "quarantine-detail",
                "m3rs:quarantine-selection",
                quarantined_snapshot,
            )
            .expect("quarantine remains readable but not continuable");
        assert_eq!(
            quarantined_detail.continuation.reason.as_deref(),
            Some("SESSION_QUARANTINED"),
        );
    }
}
