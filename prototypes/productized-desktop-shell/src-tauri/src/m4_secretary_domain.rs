//! M4C02 ordinary-product Secretary composition.
//!
//! This module is the only bridge from the fixed M4 personal identity into
//! the M3 RoleSession repository/read runtime. It accepts one backend-resolved
//! app-data root, never a renderer path, cwd, project root, or acceptance
//! permit. Creating or restoring the RoleSession records local M3 ledger rows
//! only; this module does not claim or dispatch the registered provider effect.

use crate::m3_role_session::{
    CorrelationId, OpaqueRef, PermissionSnapshotDescriptor, RequestIdempotencyKey, RoleSessionId,
    RoleSessionState, ServerResolvedBinding, Sha256Digest,
};
use crate::m3_role_session_read_model::{
    M3OrdinarySecretaryReadBinding, M3RoleSessionReadRuntimeSlot, M3SecretaryReadHost,
};
use crate::m3_role_session_repository::{
    CreateRoleSessionCommand, M3CommandMetadata, M3OrdinaryRoleSessionRepositoryConfig,
    M3ReadPermissionDisposition, M3RoleSessionDirectoryQuery, M3RoleSessionSnapshotQuery,
    M3RoleSessionSqliteRepository, M3SessionBindingReadState, QuarantineRoleSessionCommand,
    ResumeRoleSessionCommand, M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
};
use crate::mcp::identity_kernel::{
    resolve_m4_primary_secretary_identity, M4PrimarySecretaryIdentity,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

const M4_SECRETARY_ROLE_SESSION_ID_MATERIAL: &str =
    "syn.m4.secretary-role-session/personal-primary/v1";
const M4_SECRETARY_CREATE_MATERIAL: &str =
    "syn.m4.secretary-role-session-create/personal-primary/v1";

pub(crate) fn install_ordinary_product_secretary_runtime(
    app_data_root: &Path,
) -> Result<M3RoleSessionReadRuntimeSlot, String> {
    let canonical_app_data_root = admit_server_app_data_root(app_data_root)?;
    let repository = M3RoleSessionSqliteRepository::open_ordinary_product(
        &M3OrdinaryRoleSessionRepositoryConfig {
            db_path: canonical_app_data_root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
            app_data_root: canonical_app_data_root,
        },
    )
    .map_err(|error| error.code)?;
    let identity =
        resolve_m4_primary_secretary_identity().map_err(|error| error.code().to_string())?;
    let binding = identity
        .m3_server_resolved_binding()
        .map_err(|error| error.code().to_string())?;
    let role_session_id =
        bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)?;

    M3RoleSessionReadRuntimeSlot::from_ordinary_product_secretary(M3OrdinarySecretaryReadBinding {
        host: M3SecretaryReadHost::server_fixed(),
        repository,
        binding,
        role_session_id,
    })
}

/// Resolve the app-data root before any store path is constructed. The caller
/// is app composition code, not IPC; the root must already be an absolute,
/// clean server path. A symlinked alias is rejected rather than normalized.
fn admit_server_app_data_root(app_data_root: &Path) -> Result<PathBuf, String> {
    if !app_data_root.is_absolute()
        || app_data_root
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err("m4_secretary_app_data_root_clean_absolute_required".to_string());
    }
    fs::create_dir_all(app_data_root)
        .map_err(|_| "m4_secretary_app_data_root_create_failed".to_string())?;
    let canonical = fs::canonicalize(app_data_root)
        .map_err(|_| "m4_secretary_app_data_root_unavailable".to_string())?;
    if canonical != app_data_root {
        return Err("m4_secretary_app_data_root_identity_changed".to_string());
    }
    Ok(canonical)
}

fn bootstrap_or_restore_secretary_role_session(
    repository: &M3RoleSessionSqliteRepository,
    identity: &M4PrimarySecretaryIdentity,
    binding: &ServerResolvedBinding,
) -> Result<RoleSessionId, String> {
    // Directory order is only pagination mechanics. Resolve the complete
    // server-authorized set before choosing anything, so startup cannot turn
    // recency into identity authority.
    let entries = list_secretary_role_session_candidates(repository, binding)?;
    let live_candidates = entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.session.status,
                RoleSessionState::Created | RoleSessionState::Active | RoleSessionState::Suspended
            )
        })
        .collect::<Vec<_>>();
    let has_mismatched_candidate = live_candidates
        .iter()
        .any(|entry| !matches!(&entry.permission, M3ReadPermissionDisposition::Current));
    if live_candidates.len() > 1 || has_mismatched_candidate {
        for entry in live_candidates {
            quarantine_secretary_role_session_candidate(
                repository,
                binding,
                entry.session.role_session_id.clone(),
                entry.session.revision,
            )?;
        }
        return Err(if has_mismatched_candidate {
            "m4_secretary_role_session_mismatched"
        } else {
            "m4_secretary_role_session_ambiguous"
        }
        .to_string());
    }

    let Some(entry) = entries.iter().find(|entry| {
        matches!(
            entry.session.status,
            RoleSessionState::Created | RoleSessionState::Active | RoleSessionState::Suspended
        )
    }) else {
        if entries
            .iter()
            .any(|entry| entry.session.status == RoleSessionState::Quarantined)
        {
            return Err("m4_secretary_role_session_quarantined".to_string());
        }
        if entries
            .iter()
            .any(|entry| entry.session.status == RoleSessionState::Closed)
        {
            return Err("m4_secretary_role_session_closed".to_string());
        }
        return create_secretary_role_session(repository, binding)
            .map(|outcome| outcome.role_session_id);
    };
    match entry.session.status {
        RoleSessionState::Active => {
            if !matches!(&entry.permission, M3ReadPermissionDisposition::Current) {
                return Err("m4_secretary_permission_revalidation_required".to_string());
            }
            Ok(entry.session.role_session_id.clone())
        }
        RoleSessionState::Suspended => {
            if !matches!(&entry.permission, M3ReadPermissionDisposition::Current) {
                return Err("m4_secretary_permission_revalidation_required".to_string());
            }
            let snapshot = repository
                .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                    role_session_id: entry.session.role_session_id.clone(),
                    binding: binding.clone(),
                })
                .map_err(|error| error.code)?
                .ok_or_else(|| "m4_secretary_resume_session_missing".to_string())?;
            if !matches!(
                snapshot.current_binding,
                M3SessionBindingReadState::Verified { .. }
            ) {
                return Err("m4_secretary_resume_binding_unavailable".to_string());
            }
            let permission = permission_descriptor(identity, binding)?;
            let revision = entry.session.revision;
            let material = format!(
                "syn.m4.secretary-role-session-resume/personal-primary/v1/revision:{revision}"
            );
            let outcome = repository
                .resume_role_session(&ResumeRoleSessionCommand {
                    role_session_id: entry.session.role_session_id.clone(),
                    binding: binding.clone(),
                    previous_permission: Some(permission.clone()),
                    current_permission: Some(permission),
                    expected_session_revision: revision,
                    metadata: metadata_for(repository, "resume", &material)?,
                })
                .map_err(|error| error.code)?;
            let session = outcome
                .role_session
                .ok_or_else(|| "m4_secretary_resume_session_missing".to_string())?;
            if session.status != RoleSessionState::Active {
                return Err("m4_secretary_resume_not_active".to_string());
            }
            Ok(session.role_session_id)
        }
        RoleSessionState::Quarantined => Err("m4_secretary_role_session_quarantined".to_string()),
        RoleSessionState::Closed => Err("m4_secretary_role_session_closed".to_string()),
        RoleSessionState::Created => Err("m4_secretary_role_session_incomplete".to_string()),
    }
}

fn list_secretary_role_session_candidates(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
) -> Result<Vec<crate::m3_role_session_repository::M3RoleSessionDirectoryEntry>, String> {
    let mut entries = Vec::new();
    let mut after = None;
    loop {
        let page = repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding: binding.clone(),
                after: after.clone(),
                limit: 100,
            })
            .map_err(|error| error.code)?;
        entries.extend(page.entries);
        let Some(next_cursor) = page.next_cursor else {
            return Ok(entries);
        };
        after = Some(next_cursor);
    }
}

fn quarantine_secretary_role_session_candidate(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
    role_session_id: RoleSessionId,
    expected_session_revision: u64,
) -> Result<(), String> {
    let material = format!(
        "syn.m4.secretary-role-session-quarantine/personal-primary/v1/role-session:{}/revision:{expected_session_revision}",
        role_session_id.as_str(),
    );
    let outcome = repository
        .quarantine_role_session(&QuarantineRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            expected_session_revision,
            metadata: metadata_for(repository, "quarantine", &material)?,
        })
        .map_err(|_| "m4_secretary_role_session_quarantine_failed".to_string())?;
    let session = outcome
        .role_session
        .ok_or_else(|| "m4_secretary_role_session_quarantine_missing".to_string())?;
    if session.role_session_id != role_session_id || session.status != RoleSessionState::Quarantined
    {
        return Err("m4_secretary_role_session_quarantine_invalid".to_string());
    }
    Ok(())
}

struct SecretaryCreateOutcome {
    role_session_id: RoleSessionId,
    replayed: bool,
}

fn create_secretary_role_session(
    repository: &M3RoleSessionSqliteRepository,
    binding: &ServerResolvedBinding,
) -> Result<SecretaryCreateOutcome, String> {
    let role_session_id = role_session_id()?;
    let outcome = repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            metadata: metadata_for(repository, "create", M4_SECRETARY_CREATE_MATERIAL)?,
        })
        .map_err(|error| error.code)?;
    let session = outcome
        .role_session
        .ok_or_else(|| "m4_secretary_create_session_missing".to_string())?;
    if session.role_session_id != role_session_id || session.status != RoleSessionState::Active {
        return Err("m4_secretary_create_session_invalid".to_string());
    }
    Ok(SecretaryCreateOutcome {
        role_session_id,
        replayed: outcome.replayed,
    })
}

fn permission_descriptor(
    identity: &M4PrimarySecretaryIdentity,
    binding: &ServerResolvedBinding,
) -> Result<PermissionSnapshotDescriptor, String> {
    let refs = |namespace: &str, values: &[String]| -> Result<BTreeSet<OpaqueRef>, String> {
        values
            .iter()
            .map(|value| opaque_ref(namespace, value))
            .collect()
    };
    Ok(PermissionSnapshotDescriptor {
        snapshot_ref: binding.permission_snapshot_ref.clone(),
        allowed_capability_refs: refs(
            "capability",
            &identity.permission_profile.allow_capabilities,
        )?,
        denied_capability_refs: refs("capability", &identity.permission_profile.deny_capabilities)?,
        constraint_refs: refs("constraint", &identity.permission_profile.constraints)?,
    })
}

fn role_session_id() -> Result<RoleSessionId, String> {
    RoleSessionId::try_from_canonical(sealed_ref("session", M4_SECRETARY_ROLE_SESSION_ID_MATERIAL))
        .map_err(|_| "m4_secretary_role_session_id_invalid".to_string())
}

fn metadata_for(
    repository: &M3RoleSessionSqliteRepository,
    operation: &str,
    material: &str,
) -> Result<M3CommandMetadata, String> {
    Ok(M3CommandMetadata {
        receipt_id: opaque_ref("receipt", &format!("{material}/receipt"))?,
        event_id: opaque_ref("event", &format!("{material}/event"))?,
        audit_id: opaque_ref("audit", &format!("{material}/audit"))?,
        correlation_id: CorrelationId::try_from_canonical(sealed_ref(
            "correlation",
            &format!("{material}/correlation"),
        ))
        .map_err(|_| "m4_secretary_correlation_id_invalid".to_string())?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_ref(
            "request",
            &format!("{material}/idempotency/{operation}"),
        ))
        .map_err(|_| "m4_secretary_idempotency_key_invalid".to_string())?,
        occurred_at: repository
            .capture_server_utc_now()
            .map_err(|_| "m4_secretary_server_clock_unavailable".to_string())?,
    })
}

fn opaque_ref(namespace: &str, material: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed_ref(namespace, material))
        .map_err(|_| "m4_secretary_opaque_ref_invalid".to_string())
}

fn sealed_ref(namespace: &str, material: &str) -> String {
    format!(
        "{namespace}:sha256:{}",
        Sha256Digest::of_bytes(material.as_bytes()).as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct AppDataFixture {
        fixture_root: PathBuf,
        root: PathBuf,
    }

    impl AppDataFixture {
        fn new(label: &str) -> Self {
            let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let fixture_root = std::env::temp_dir().join(format!(
                "syn-m4c02-domain-{label}-{}-{sequence}",
                std::process::id()
            ));
            let requested = fixture_root.join("local.codex.governance.workbench");
            fs::create_dir_all(&requested).expect("create M4C02 app-data fixture");
            let root = fs::canonicalize(requested).expect("canonical M4C02 app-data fixture");
            Self { fixture_root, root }
        }

        fn repository(&self) -> M3RoleSessionSqliteRepository {
            M3RoleSessionSqliteRepository::open_ordinary_product(
                &M3OrdinaryRoleSessionRepositoryConfig {
                    app_data_root: self.root.clone(),
                    db_path: self.root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
                },
            )
            .expect("open M4C02 ordinary repository fixture")
        }
    }

    impl Drop for AppDataFixture {
        fn drop(&mut self) {
            let db_path = self.root.join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
            for suffix in ["", "-wal", "-shm"] {
                let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", db_path.display())));
            }
            let _ = fs::remove_dir_all(&self.fixture_root);
        }
    }

    #[test]
    fn m4c02_bootstrap_is_idempotent_and_restart_reloads_one_active_session() {
        let fixture = AppDataFixture::new("restart");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        repository
            .set_test_server_utc_now("2026-08-10T12:00:00Z")
            .expect("set deterministic M3 server clock");

        let first = create_secretary_role_session(&repository, &binding)
            .expect("first deterministic create");
        repository
            .set_test_server_utc_now("2026-08-10T12:01:00Z")
            .expect("advance deterministic M3 server clock");
        let replay = create_secretary_role_session(&repository, &binding)
            .expect("exact deterministic create replay");
        assert!(!first.replayed);
        assert!(replay.replayed);
        assert_eq!(first.role_session_id, replay.role_session_id);

        let first_runtime = install_ordinary_product_secretary_runtime(&fixture.root)
            .expect("first ordinary runtime install");
        let first_status = first_runtime.secretary_status().expect("first status");
        drop(first_runtime);
        let restarted_runtime = install_ordinary_product_secretary_runtime(&fixture.root)
            .expect("restart ordinary runtime install");
        let restarted_status = restarted_runtime
            .secretary_status()
            .expect("restarted status");
        assert_eq!(first_status, restarted_status);
        assert_eq!(restarted_status.session_state, "ACTIVE");

        let page = fixture
            .repository()
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding,
                after: None,
                limit: 100,
            })
            .expect("list exact Secretary sessions");
        assert_eq!(page.entries.len(), 1);
        assert!(page.next_cursor.is_none());
        assert_eq!(page.entries[0].session.created_at, "2026-08-10T12:00:00Z");
    }

    #[test]
    fn m4c02_bootstrap_keeps_unbound_suspended_session_fail_closed() {
        let fixture = AppDataFixture::new("resume");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        let created = create_secretary_role_session(&repository, &binding)
            .expect("create primary Secretary session");
        let previous_permission =
            permission_descriptor(&identity, &binding).expect("current permission descriptor");
        let wider_snapshot = opaque_ref("permission", "m4c02-wider-snapshot")
            .expect("wider permission snapshot ref");
        let wider_binding = ServerResolvedBinding::from_server_canonical(
            binding.actor_id.as_str().to_string(),
            binding.role_ref.as_str().to_string(),
            binding.scope_ref.as_str().to_string(),
            binding.current_object_ref.as_str().to_string(),
            binding.execution_channel.as_str().to_string(),
            wider_snapshot.as_str().to_string(),
        )
        .expect("same-owner wider binding");
        let mut wider_permission = previous_permission.clone();
        wider_permission.snapshot_ref = wider_snapshot;
        wider_permission
            .allowed_capability_refs
            .insert(opaque_ref("capability", "m4c02-wider-capability").expect("wider cap"));
        let active = repository
            .load_authorized_role_session_snapshot(
                &crate::m3_role_session_repository::M3RoleSessionSnapshotQuery {
                    role_session_id: created.role_session_id.clone(),
                    binding: binding.clone(),
                },
            )
            .expect("load active session")
            .expect("active session exists");
        let suspended = repository
            .resume_role_session(&ResumeRoleSessionCommand {
                role_session_id: created.role_session_id.clone(),
                binding: wider_binding,
                previous_permission: Some(previous_permission),
                current_permission: Some(wider_permission),
                expected_session_revision: active.session.revision,
                metadata: metadata_for(
                    &repository,
                    "resume",
                    "syn.m4.secretary-role-session-resume/wider-fixture/v1",
                )
                .expect("wider resume metadata"),
            })
            .expect("wider request records a suspended outcome")
            .role_session
            .expect("suspended session outcome");
        assert_eq!(suspended.status, RoleSessionState::Suspended);

        let error = bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)
            .expect_err("an unbound suspended session needs verified binding evidence");
        assert_eq!(error, "m4_secretary_resume_binding_unavailable");
        let snapshot = repository
            .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
                role_session_id: created.role_session_id,
                binding,
            })
            .expect("load fail-closed session")
            .expect("fail-closed session exists");
        assert_eq!(snapshot.session.status, RoleSessionState::Suspended);
    }

    #[test]
    fn m4c02_bootstrap_rejects_ambiguous_exact_owner_without_recency_selection() {
        let fixture = AppDataFixture::new("ambiguous");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        let first = create_secretary_role_session(&repository, &binding).expect("primary session");
        let second_role_session_id = RoleSessionId::try_from_canonical(sealed_ref(
            "session",
            "syn.m4.secretary-role-session/ambiguous-fixture/v1",
        ))
        .expect("second session id");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: second_role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m4.secretary-role-session-create/ambiguous-fixture/v1",
                )
                .expect("second create metadata"),
            })
            .expect("create second exact-owner session");

        assert_eq!(
            bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)
                .expect_err("multiple exact sessions must fail closed"),
            "m4_secretary_role_session_ambiguous",
        );
        let quarantined = repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding,
                after: None,
                limit: 100,
            })
            .expect("list quarantined ambiguous candidates");
        assert_eq!(quarantined.entries.len(), 2);
        assert!(quarantined.entries.iter().all(|entry| {
            entry.session.status == RoleSessionState::Quarantined
                && entry.session.resolution_reason
                    == Some(
                        crate::m3_role_session::SessionResolutionReason::OwnerScopeOrHandleMappingAmbiguous,
                    )
        }));
        assert!(quarantined
            .entries
            .iter()
            .any(|entry| entry.session.role_session_id == first.role_session_id));
        assert!(quarantined
            .entries
            .iter()
            .any(|entry| entry.session.role_session_id == second_role_session_id));
    }

    #[test]
    fn m4c02_bootstrap_quarantines_mismatched_secretary_candidate() {
        let fixture = AppDataFixture::new("mismatched");
        let identity = resolve_m4_primary_secretary_identity().expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        let repository = fixture.repository();
        let mismatched_snapshot = opaque_ref("permission", "m4c02-mismatched-snapshot")
            .expect("mismatched permission snapshot");
        let mismatched_binding = ServerResolvedBinding::from_server_canonical(
            binding.actor_id.as_str().to_string(),
            binding.role_ref.as_str().to_string(),
            binding.scope_ref.as_str().to_string(),
            binding.current_object_ref.as_str().to_string(),
            binding.execution_channel.as_str().to_string(),
            mismatched_snapshot.as_str().to_string(),
        )
        .expect("same-owner mismatched binding");
        let role_session_id = RoleSessionId::try_from_canonical(sealed_ref(
            "session",
            "syn.m4.secretary-role-session/mismatched-fixture/v1",
        ))
        .expect("mismatched candidate id");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: mismatched_binding,
                metadata: metadata_for(
                    &repository,
                    "create",
                    "syn.m4.secretary-role-session-create/mismatched-fixture/v1",
                )
                .expect("mismatched create metadata"),
            })
            .expect("create mismatched candidate");

        assert_eq!(
            bootstrap_or_restore_secretary_role_session(&repository, &identity, &binding)
                .expect_err("a permission-mismatched candidate must never be reused"),
            "m4_secretary_role_session_mismatched",
        );
        let quarantined = repository
            .list_authorized_role_session_directory(&M3RoleSessionDirectoryQuery {
                binding,
                after: None,
                limit: 100,
            })
            .expect("list quarantined mismatched candidate");
        assert_eq!(quarantined.entries.len(), 1);
        assert_eq!(
            quarantined.entries[0].session.role_session_id,
            role_session_id
        );
        assert_eq!(
            quarantined.entries[0].session.status,
            RoleSessionState::Quarantined
        );
    }

    #[test]
    fn m4c02_app_data_root_alias_fails_before_database_creation() {
        let fixture = AppDataFixture::new("root-alias");
        let alias = fixture.root.join("child/..");
        let error = match install_ordinary_product_secretary_runtime(&alias) {
            Ok(_) => panic!("unclean server path must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error, "m4_secretary_app_data_root_clean_absolute_required",);
        assert!(!fixture
            .root
            .join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH)
            .exists());
    }

    #[test]
    fn m4c02_product_composition_uses_tauri_app_data_root_before_app_state_install() {
        let entrypoints = include_str!("index_host_app_entrypoints.rs");
        let tauri_root = entrypoints
            .find(".app_data_dir()")
            .expect("ordinary composition resolves Tauri app-data root");
        let ordinary_constructor = entrypoints
            .find("AppState::try_new_with_tauri_app_data_root(&app_data_root)")
            .expect("ordinary AppState receives the Tauri-resolved root");
        let setup = entrypoints
            .find(".setup(move |app| {")
            .expect("Tauri setup owns ordinary product composition");
        let acceptance_constructor = entrypoints
            .find("AppState::try_new()")
            .expect("acceptance profile constructs its isolated AppState");
        assert!(tauri_root < ordinary_constructor);
        assert!(acceptance_constructor < setup);
        assert!(ordinary_constructor > setup);
        assert_eq!(entrypoints.matches("AppState::try_new()").count(), 1);
        assert_eq!(
            entrypoints
                .matches("AppState::try_new_with_tauri_app_data_root(&app_data_root)")
                .count(),
            1
        );

        let lib = include_str!("lib.rs");
        assert!(lib.contains("fn try_new_with_tauri_app_data_root(app_data_root: &Path)"));
        assert!(lib.contains("m3_role_session_read_runtime: Default::default()"));
        assert!(!entrypoints.contains(
            "AppState::try_new_with_tauri_app_data_root(&default_workflow_state_path())"
        ));
    }
}
