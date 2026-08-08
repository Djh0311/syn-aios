// SYN-M2A-T4: server-owned execution grants for the legacy dispatch ledger.
//
// This module is deliberately narrow. An immutable grant is persisted inside
// the canonical `workflow_node_dispatches` record before the runner starts and
// is reloaded from that record before a worker report can be stored. It is not
// a substitute for the M3 RoleSession / PreparedAttempt model.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

const GRANT_SCHEMA_VERSION: &str = "execution-grant-ledger.v2";
const MAX_TTL_SECONDS: u64 = 24 * 60 * 60;

/// Unique, server-random identifier. It is never derived from a dispatch id.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct GrantId(pub(crate) String);

/// Immutable authorization material stored in a server-owned dispatch record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ExecutionGrant {
    pub(crate) grant_id: GrantId,
    pub(crate) schema_version: String,
    pub(crate) authorization_id: String,
    pub(crate) authorization_store_revision: i64,
    /// Hash of the exact authoritative authorization record used at reservation
    /// time.  It makes a DB-primary re-read distinguish a stale JSON sidecar
    /// from the current source even when a caller happens to retain an older
    /// store revision.
    pub(crate) authorization_source_hash: Option<String>,
    pub(crate) scope_fingerprint: String,
    pub(crate) principal: String,
    pub(crate) project_id: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow_node_id: Option<String>,
    pub(crate) work_item_id: Option<String>,
    pub(crate) dispatch_id: Option<String>,
    pub(crate) attempt_id: Option<String>,
    pub(crate) binding_id: Option<String>,
    /// The C4 record consumed by this dispatch.  It is intentionally M2
    /// dispatch bookkeeping, not an M5 PreparedAttempt claim.
    pub(crate) prepared_dispatch_id: Option<String>,
    pub(crate) max_worker_dispatches: Option<i64>,
    pub(crate) max_runtime_minutes: Option<i64>,
    pub(crate) allowed_work_item_types: Vec<String>,
    pub(crate) allowed_role_ids: Vec<String>,
    pub(crate) allowed_agent_ids: Vec<String>,
    pub(crate) allowed_read_roots: Vec<String>,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) allowed_tools: Vec<String>,
    pub(crate) allowed_checks: Vec<String>,
    pub(crate) stop_conditions: Vec<String>,
    pub(crate) minted_at_ms: i64,
    pub(crate) expires_at_ms: i64,
    pub(crate) revoked_at_ms: Option<i64>,
    pub(crate) revoked_reason: Option<String>,
    pub(crate) grant_hash: String,
    pub(crate) minted_by: String,
}

impl ExecutionGrant {
    pub(crate) fn is_valid(&self) -> bool {
        self.revoked_at_ms.is_none()
            && self.expires_at_ms > unix_timestamp_ms()
            && verify_integrity(self).is_ok()
    }

    pub(crate) fn covers_project(&self, project_id: &str) -> bool {
        self.project_id == project_id
    }

    pub(crate) fn covers_workflow(&self, workflow_id: &str) -> bool {
        self.workflow_id == workflow_id
    }

    pub(crate) fn allows_role(&self, role_id: &str) -> bool {
        self.allowed_role_ids.iter().any(|value| value == role_id)
    }

    pub(crate) fn allows_agent(&self, agent_id: &str) -> bool {
        // Match the canonical plan-authorization contract: an empty agent
        // list means the authorization has no extra agent filter.  The grant
        // is still bound to one exact persisted principal below, so this does
        // not turn a dispatch grant into a bearer credential.
        self.allowed_agent_ids.is_empty()
            || self.allowed_agent_ids.iter().any(|value| value == agent_id)
    }

    pub(crate) fn allows_tool(&self, tool_id: &str) -> bool {
        self.allowed_tools.iter().any(|value| value == tool_id)
    }

    pub(crate) fn allows_write_root(&self, root: &str) -> bool {
        self.allowed_write_roots.iter().any(|value| value == root)
    }

    pub(crate) fn allows_read_root(&self, root: &str) -> bool {
        self.allowed_read_roots.iter().any(|value| value == root)
    }
}

/// Test-only minting input.  Production dispatches never deserialize or accept
/// caller-supplied grant scope; they mint exclusively from the active plan
/// authorization store through `mint_dispatch_grant`.
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GrantMintInput {
    pub authorization_id: String,
    pub authorization_revision: u64,
    pub scope_fingerprint: String,
    pub principal: String,
    pub project_id: String,
    pub workflow_id: String,
    pub allowed_work_item_types: Vec<String>,
    pub allowed_role_ids: Vec<String>,
    pub allowed_agent_ids: Vec<String>,
    pub allowed_read_roots: Vec<String>,
    pub allowed_write_roots: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub allowed_checks: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub ttl_seconds: u64,
    pub minted_by: String,
}

/// Internal material used by the production-only dispatch minting path and by
/// test fixtures.  It is deliberately private so no product caller can mint a
/// grant from an arbitrary supplied scope.
struct GrantMaterial<'a> {
    authorization_id: &'a str,
    principal: &'a str,
    project_id: &'a str,
    workflow_id: &'a str,
    allowed_work_item_types: &'a [String],
    allowed_role_ids: &'a [String],
    allowed_agent_ids: &'a [String],
    allowed_read_roots: &'a [String],
    allowed_write_roots: &'a [String],
    allowed_tools: &'a [String],
    allowed_checks: &'a [String],
    stop_conditions: &'a [String],
    minted_by: &'a str,
}

/// Exact authorization material loaded by the server from the active plan
/// authorization store. It must never be assembled from a worker report.
/// This is a server-side capability, not a worker-provided tool name: only an
/// explicitly user-confirmed authorization containing it may enter the M2
/// grant route.  Historic M1 scopes do not contain it and remain no-grant.
pub(crate) const EXECUTION_GRANT_LEDGER_V2_CAPABILITY: &str =
    "server_execution_grant_ledger_v2";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutionGrantAuthorizationSource {
    pub authorization_id: String,
    pub authorization_store_revision: i64,
    pub authorization_source_hash: String,
    pub project_id: String,
    pub workflow_id: String,
    pub allowed_work_item_types: Vec<String>,
    pub allowed_role_ids: Vec<String>,
    pub allowed_agent_ids: Vec<String>,
    pub allowed_read_roots: Vec<String>,
    pub allowed_write_roots: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub allowed_checks: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub expires_at_ms: Option<i64>,
    pub max_worker_dispatches: Option<i64>,
    pub max_runtime_minutes: Option<i64>,
}

/// Immutable server context written alongside a grant in the dispatch ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutionGrantBinding {
    pub dispatch_id: String,
    pub project_id: String,
    pub workflow_id: String,
    pub workflow_node_id: String,
    pub work_item_id: String,
    pub binding_id: String,
    pub principal: String,
    pub prepared_dispatch_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DispatchGrantConstraints {
    authorization_source_hash: String,
    prepared_dispatch_id: String,
    max_worker_dispatches: i64,
    max_runtime_minutes: i64,
}

/// Context reloaded by the report consumer, not received as a worker claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DispatchGrantVerificationContext<'a> {
    pub project_id: &'a str,
    pub workflow_id: &'a str,
    pub workflow_node_id: &'a str,
    pub work_item_id: &'a str,
    pub dispatch_id: &'a str,
    pub attempt_id: &'a str,
    pub binding_id: &'a str,
    pub principal: &'a str,
    pub actor_role: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GrantVerification {
    Valid,
    Expired,
    Revoked {
        reason: String,
    },
    ProjectMismatch {
        expected: String,
        actual: String,
    },
    WorkflowMismatch {
        expected: String,
        actual: String,
    },
    RoleNotAllowed {
        role: String,
    },
    AgentNotAllowed {
        agent: String,
    },
    ToolNotAllowed {
        tool: String,
    },
    WriteRootDenied {
        root: String,
    },
    IntegrityMismatch {
        expected: String,
        actual: String,
    },
    InvalidGrant {
        reason: String,
    },
    BindingMismatch {
        field: String,
        expected: String,
        actual: String,
    },
}

/// Mints a non-persisted grant for focused unit fixtures. Production dispatches
/// use `mint_dispatch_grant`, which binds a fresh random grant to a persisted
/// server dispatch record.
#[cfg(test)]
pub(crate) fn mint_grant(input: &GrantMintInput) -> Result<ExecutionGrant, String> {
    let now = unix_timestamp_ms();
    let expires_at_ms = checked_expiry(now, input.ttl_seconds, None)?;
    let material = GrantMaterial {
        authorization_id: &input.authorization_id,
        principal: &input.principal,
        project_id: &input.project_id,
        workflow_id: &input.workflow_id,
        allowed_work_item_types: &input.allowed_work_item_types,
        allowed_role_ids: &input.allowed_role_ids,
        allowed_agent_ids: &input.allowed_agent_ids,
        allowed_read_roots: &input.allowed_read_roots,
        allowed_write_roots: &input.allowed_write_roots,
        allowed_tools: &input.allowed_tools,
        allowed_checks: &input.allowed_checks,
        stop_conditions: &input.stop_conditions,
        minted_by: &input.minted_by,
    };
    build_grant(
        &material,
        i64::try_from(input.authorization_revision)
            .map_err(|_| "execution_grant_authorization_revision_overflow".to_string())?,
        None,
        now,
        expires_at_ms,
    )
}

/// Mints the M2 legacy-dispatch ledger record. The source has already been
/// freshly loaded and validated by `plan_authorization_store`.
pub(crate) fn mint_dispatch_grant(
    source: &ExecutionGrantAuthorizationSource,
    binding: &ExecutionGrantBinding,
    ttl_seconds: u64,
) -> Result<ExecutionGrant, String> {
    if source.authorization_store_revision < 0 {
        return Err("execution_grant_authorization_revision_invalid".to_string());
    }
    if source.project_id != binding.project_id || source.workflow_id != binding.workflow_id {
        return Err("execution_grant_authorization_scope_mismatch".to_string());
    }
    let material = GrantMaterial {
        authorization_id: &source.authorization_id,
        principal: &binding.principal,
        project_id: &binding.project_id,
        workflow_id: &binding.workflow_id,
        allowed_work_item_types: &source.allowed_work_item_types,
        allowed_role_ids: &source.allowed_role_ids,
        allowed_agent_ids: &source.allowed_agent_ids,
        allowed_read_roots: &source.allowed_read_roots,
        allowed_write_roots: &source.allowed_write_roots,
        allowed_tools: &source.allowed_tools,
        allowed_checks: &source.allowed_checks,
        stop_conditions: &source.stop_conditions,
        minted_by: "server:m2-legacy-dispatch-grant-ledger",
    };
    let max_worker_dispatches = source
        .max_worker_dispatches
        .filter(|value| *value > 0)
        .ok_or_else(|| "execution_grant_worker_quota_missing_or_invalid".to_string())?;
    let max_runtime_minutes = source
        .max_runtime_minutes
        .filter(|value| *value > 0)
        .ok_or_else(|| "execution_grant_runtime_quota_missing_or_invalid".to_string())?;
    if source.authorization_source_hash.trim().is_empty() {
        return Err("execution_grant_authorization_source_hash_missing".to_string());
    }
    if binding.prepared_dispatch_id.trim().is_empty() {
        return Err("execution_grant_prepared_dispatch_id_missing".to_string());
    }
    let runtime_seconds = u64::try_from(max_runtime_minutes)
        .map_err(|_| "execution_grant_runtime_quota_invalid".to_string())?
        .checked_mul(60)
        .ok_or_else(|| "execution_grant_runtime_quota_overflow".to_string())?;
    // The source's runtime budget is the authorization limit.  A caller may
    // request a narrower budget but never a broader or fixed fallback TTL.
    let ttl_seconds = ttl_seconds.min(runtime_seconds);
    let now = unix_timestamp_ms();
    let expires_at_ms = checked_expiry(now, ttl_seconds, source.expires_at_ms)?;
    let attempt_id = format!(
        "attempt:{}",
        crate::utils::hash::sha256_hex(&binding.dispatch_id)
    );
    let binding = ExecutionGrantBinding {
        dispatch_id: binding.dispatch_id.clone(),
        project_id: binding.project_id.clone(),
        workflow_id: binding.workflow_id.clone(),
        workflow_node_id: binding.workflow_node_id.clone(),
        work_item_id: binding.work_item_id.clone(),
        binding_id: binding.binding_id.clone(),
        principal: binding.principal.clone(),
        prepared_dispatch_id: binding.prepared_dispatch_id.clone(),
    };
    let prepared_dispatch_id = binding.prepared_dispatch_id.clone();
    let grant = build_grant(
        &material,
        source.authorization_store_revision,
        Some((
            binding,
            attempt_id,
            DispatchGrantConstraints {
                authorization_source_hash: source.authorization_source_hash.clone(),
                prepared_dispatch_id: source_authorized_prepared_id(source, &prepared_dispatch_id)?,
                max_worker_dispatches,
                max_runtime_minutes,
            },
        )),
        now,
        expires_at_ms,
    )?;
    if grant.dispatch_id.as_deref() == Some(grant.grant_id.0.as_str())
        || grant.attempt_id.as_deref() == Some(grant.grant_id.0.as_str())
    {
        return Err("execution_grant_identifier_alias_rejected".to_string());
    }
    Ok(grant)
}

fn source_authorized_prepared_id(
    source: &ExecutionGrantAuthorizationSource,
    prepared_dispatch_id: &str,
) -> Result<String, String> {
    if prepared_dispatch_id.trim().is_empty() || source.authorization_source_hash.trim().is_empty()
    {
        return Err("execution_grant_prepared_source_invalid".to_string());
    }
    Ok(prepared_dispatch_id.to_string())
}

#[cfg(test)]
pub(crate) fn revoke_grant(grant: &mut ExecutionGrant, reason: &str) {
    grant.revoked_at_ms = Some(unix_timestamp_ms());
    grant.revoked_reason = Some(reason.to_string());
    grant.grant_hash = grant_hash(grant);
}

pub(crate) fn verify_grant(
    grant: &ExecutionGrant,
    project_id: &str,
    workflow_id: &str,
    role_id: &str,
    agent_id: Option<&str>,
    tool_id: Option<&str>,
    write_root: Option<&str>,
) -> GrantVerification {
    if let Err(reason) = verify_integrity(grant) {
        return GrantVerification::InvalidGrant { reason };
    }
    let expected_hash = grant_hash(grant);
    if grant.grant_hash != expected_hash {
        return GrantVerification::IntegrityMismatch {
            expected: expected_hash,
            actual: grant.grant_hash.clone(),
        };
    }
    if let Some(reason) = &grant.revoked_reason {
        return GrantVerification::Revoked {
            reason: reason.clone(),
        };
    }
    if grant.expires_at_ms <= unix_timestamp_ms() {
        return GrantVerification::Expired;
    }
    if !grant.covers_project(project_id) {
        return GrantVerification::ProjectMismatch {
            expected: grant.project_id.clone(),
            actual: project_id.to_string(),
        };
    }
    if !grant.covers_workflow(workflow_id) {
        return GrantVerification::WorkflowMismatch {
            expected: grant.workflow_id.clone(),
            actual: workflow_id.to_string(),
        };
    }
    if !grant.allows_role(role_id) {
        return GrantVerification::RoleNotAllowed {
            role: role_id.to_string(),
        };
    }
    if let Some(agent) = agent_id {
        if !grant.allows_agent(agent) {
            return GrantVerification::AgentNotAllowed {
                agent: agent.to_string(),
            };
        }
    }
    if let Some(tool) = tool_id {
        if !grant.allows_tool(tool) {
            return GrantVerification::ToolNotAllowed {
                tool: tool.to_string(),
            };
        }
    }
    if let Some(root) = write_root {
        if !grant.allows_write_root(root) {
            return GrantVerification::WriteRootDenied {
                root: root.to_string(),
            };
        }
    }
    GrantVerification::Valid
}

pub(crate) fn verify_dispatch_grant(
    grant: &ExecutionGrant,
    context: &DispatchGrantVerificationContext<'_>,
) -> GrantVerification {
    let generic = verify_grant(
        grant,
        context.project_id,
        context.workflow_id,
        context.actor_role,
        Some(context.principal),
        None,
        None,
    );
    if generic != GrantVerification::Valid {
        return generic;
    }
    for (field, expected, actual) in [
        (
            "dispatch_id",
            grant.dispatch_id.as_deref(),
            Some(context.dispatch_id),
        ),
        (
            "attempt_id",
            grant.attempt_id.as_deref(),
            Some(context.attempt_id),
        ),
        (
            "workflow_node_id",
            grant.workflow_node_id.as_deref(),
            Some(context.workflow_node_id),
        ),
        (
            "work_item_id",
            grant.work_item_id.as_deref(),
            Some(context.work_item_id),
        ),
        (
            "binding_id",
            grant.binding_id.as_deref(),
            Some(context.binding_id),
        ),
        (
            "principal",
            Some(grant.principal.as_str()),
            Some(context.principal),
        ),
    ] {
        if expected != actual {
            return GrantVerification::BindingMismatch {
                field: field.to_string(),
                expected: expected.unwrap_or("<missing>").to_string(),
                actual: actual.unwrap_or("<missing>").to_string(),
            };
        }
    }
    GrantVerification::Valid
}

/// Rechecks that a persisted M2 dispatch grant still corresponds to the exact
/// active authorization source.  A store revision change is deliberately
/// fail-closed: it may represent revocation or a scope change, and an old
/// dispatch grant must never silently outlive either one.
pub(crate) fn verify_dispatch_grant_authorization_source(
    grant: &ExecutionGrant,
    source: &ExecutionGrantAuthorizationSource,
) -> Result<(), String> {
    if grant.authorization_id != source.authorization_id {
        return Err("execution_grant_source_authorization_id_mismatch".to_string());
    }
    if grant.authorization_store_revision != source.authorization_store_revision {
        return Err("execution_grant_source_revision_mismatch".to_string());
    }
    if grant.authorization_source_hash.as_deref() != Some(source.authorization_source_hash.as_str())
    {
        return Err("execution_grant_source_hash_mismatch".to_string());
    }
    if grant.max_worker_dispatches != source.max_worker_dispatches
        || grant.max_runtime_minutes != source.max_runtime_minutes
    {
        return Err("execution_grant_source_quota_mismatch".to_string());
    }
    if grant.project_id != source.project_id || grant.workflow_id != source.workflow_id {
        return Err("execution_grant_source_project_or_workflow_mismatch".to_string());
    }
    let normalize = |field: &str, values: &[String]| {
        normalized_scope(field, values).map_err(|reason| format!("execution_grant_source_{reason}"))
    };
    let allowed_work_item_types =
        normalize("allowed_work_item_types", &source.allowed_work_item_types)?;
    let allowed_role_ids = normalize("allowed_role_ids", &source.allowed_role_ids)?;
    let allowed_agent_ids = normalize("allowed_agent_ids", &source.allowed_agent_ids)?;
    let allowed_read_roots = normalize("allowed_read_roots", &source.allowed_read_roots)?;
    let allowed_write_roots = normalize("allowed_write_roots", &source.allowed_write_roots)?;
    let allowed_tools = normalize("allowed_tools", &source.allowed_tools)?;
    let allowed_checks = normalize("allowed_checks", &source.allowed_checks)?;
    let stop_conditions = normalize("stop_conditions", &source.stop_conditions)?;
    for (field, actual, expected) in [
        (
            "allowed_work_item_types",
            &grant.allowed_work_item_types,
            &allowed_work_item_types,
        ),
        (
            "allowed_role_ids",
            &grant.allowed_role_ids,
            &allowed_role_ids,
        ),
        (
            "allowed_agent_ids",
            &grant.allowed_agent_ids,
            &allowed_agent_ids,
        ),
        (
            "allowed_read_roots",
            &grant.allowed_read_roots,
            &allowed_read_roots,
        ),
        (
            "allowed_write_roots",
            &grant.allowed_write_roots,
            &allowed_write_roots,
        ),
        ("allowed_tools", &grant.allowed_tools, &allowed_tools),
        ("allowed_checks", &grant.allowed_checks, &allowed_checks),
        ("stop_conditions", &grant.stop_conditions, &stop_conditions),
    ] {
        if actual != expected {
            return Err(format!("execution_grant_source_{field}_mismatch"));
        }
    }
    let expected_scope_fingerprint = scope_fingerprint(
        &source.project_id,
        &source.workflow_id,
        &allowed_work_item_types,
        &allowed_role_ids,
        &allowed_agent_ids,
        &allowed_read_roots,
        &allowed_write_roots,
        &allowed_tools,
        &allowed_checks,
        &stop_conditions,
        source.max_worker_dispatches,
        source.max_runtime_minutes,
    );
    if grant.scope_fingerprint != expected_scope_fingerprint {
        return Err("execution_grant_source_scope_fingerprint_mismatch".to_string());
    }
    if source
        .expires_at_ms
        .is_some_and(|source_expiry| grant.expires_at_ms > source_expiry)
    {
        return Err("execution_grant_source_expiry_mismatch".to_string());
    }
    Ok(())
}

fn build_grant(
    input: &GrantMaterial<'_>,
    authorization_store_revision: i64,
    binding: Option<(ExecutionGrantBinding, String, DispatchGrantConstraints)>,
    minted_at_ms: i64,
    expires_at_ms: i64,
) -> Result<ExecutionGrant, String> {
    require_nonempty("authorization_id", &input.authorization_id)?;
    require_nonempty("principal", &input.principal)?;
    require_nonempty("project_id", &input.project_id)?;
    require_nonempty("workflow_id", &input.workflow_id)?;
    require_nonempty("minted_by", &input.minted_by)?;
    if authorization_store_revision < 0 {
        return Err("execution_grant_authorization_revision_invalid".to_string());
    }
    let allowed_work_item_types =
        normalized_scope("allowed_work_item_types", &input.allowed_work_item_types)?;
    let allowed_role_ids = normalized_scope("allowed_role_ids", &input.allowed_role_ids)?;
    let allowed_agent_ids = normalized_scope("allowed_agent_ids", &input.allowed_agent_ids)?;
    let allowed_read_roots = normalized_scope("allowed_read_roots", &input.allowed_read_roots)?;
    let allowed_write_roots = normalized_scope("allowed_write_roots", &input.allowed_write_roots)?;
    let allowed_tools = normalized_scope("allowed_tools", &input.allowed_tools)?;
    let allowed_checks = normalized_scope("allowed_checks", &input.allowed_checks)?;
    let stop_conditions = normalized_scope("stop_conditions", &input.stop_conditions)?;
    let (
        dispatch_id,
        attempt_id,
        workflow_node_id,
        work_item_id,
        binding_id,
        prepared_dispatch_id,
        authorization_source_hash,
        max_worker_dispatches,
        max_runtime_minutes,
    ) = match binding {
        Some((binding, attempt_id, constraints)) => {
            require_nonempty("dispatch_id", &binding.dispatch_id)?;
            require_nonempty("attempt_id", &attempt_id)?;
            require_nonempty("workflow_node_id", &binding.workflow_node_id)?;
            require_nonempty("work_item_id", &binding.work_item_id)?;
            require_nonempty("binding_id", &binding.binding_id)?;
            require_nonempty("prepared_dispatch_id", &constraints.prepared_dispatch_id)?;
            require_nonempty(
                "authorization_source_hash",
                &constraints.authorization_source_hash,
            )?;
            if constraints.max_worker_dispatches <= 0 || constraints.max_runtime_minutes <= 0 {
                return Err("execution_grant_dispatch_quota_invalid".to_string());
            }
            if binding.principal.as_str() != input.principal
                || binding.project_id.as_str() != input.project_id
                || binding.workflow_id.as_str() != input.workflow_id
            {
                return Err("execution_grant_binding_source_mismatch".to_string());
            }
            (
                Some(binding.dispatch_id),
                Some(attempt_id),
                Some(binding.workflow_node_id),
                Some(binding.work_item_id),
                Some(binding.binding_id),
                Some(constraints.prepared_dispatch_id),
                Some(constraints.authorization_source_hash),
                Some(constraints.max_worker_dispatches),
                Some(constraints.max_runtime_minutes),
            )
        }
        None => (None, None, None, None, None, None, None, None, None),
    };
    let scope_fingerprint = scope_fingerprint(
        &input.project_id,
        &input.workflow_id,
        &allowed_work_item_types,
        &allowed_role_ids,
        &allowed_agent_ids,
        &allowed_read_roots,
        &allowed_write_roots,
        &allowed_tools,
        &allowed_checks,
        &stop_conditions,
        max_worker_dispatches,
        max_runtime_minutes,
    );
    let mut grant = ExecutionGrant {
        grant_id: GrantId(random_identifier("grant")?),
        schema_version: GRANT_SCHEMA_VERSION.to_string(),
        authorization_id: input.authorization_id.to_string(),
        authorization_store_revision,
        authorization_source_hash,
        scope_fingerprint,
        principal: input.principal.to_string(),
        project_id: input.project_id.to_string(),
        workflow_id: input.workflow_id.to_string(),
        workflow_node_id,
        work_item_id,
        dispatch_id,
        attempt_id,
        binding_id,
        prepared_dispatch_id,
        max_worker_dispatches,
        max_runtime_minutes,
        allowed_work_item_types,
        allowed_role_ids,
        allowed_agent_ids,
        allowed_read_roots,
        allowed_write_roots,
        allowed_tools,
        allowed_checks,
        stop_conditions,
        minted_at_ms,
        expires_at_ms,
        revoked_at_ms: None,
        revoked_reason: None,
        grant_hash: String::new(),
        minted_by: input.minted_by.to_string(),
    };
    grant.grant_hash = grant_hash(&grant);
    Ok(grant)
}

fn checked_expiry(
    now_ms: i64,
    ttl_seconds: u64,
    source_expiry_ms: Option<i64>,
) -> Result<i64, String> {
    if ttl_seconds == 0 || ttl_seconds > MAX_TTL_SECONDS {
        return Err("execution_grant_ttl_out_of_range".to_string());
    }
    let ttl_ms = i64::try_from(ttl_seconds)
        .map_err(|_| "execution_grant_ttl_overflow".to_string())?
        .checked_mul(1_000)
        .ok_or_else(|| "execution_grant_ttl_overflow".to_string())?;
    let local_expiry = now_ms
        .checked_add(ttl_ms)
        .ok_or_else(|| "execution_grant_expiry_overflow".to_string())?;
    let expires_at_ms = source_expiry_ms
        .map(|value| value.min(local_expiry))
        .unwrap_or(local_expiry);
    if expires_at_ms <= now_ms {
        return Err("execution_grant_source_expired".to_string());
    }
    Ok(expires_at_ms)
}

fn require_nonempty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("execution_grant_{field}_missing"))
    } else {
        Ok(())
    }
}

fn normalized_scope(field: &str, values: &[String]) -> Result<Vec<String>, String> {
    let mut normalized = values
        .iter()
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    if normalized.iter().any(|value| value.is_empty()) {
        return Err(format!("execution_grant_{field}_contains_empty"));
    }
    if normalized.iter().any(|value| value == "*") {
        return Err(format!("execution_grant_{field}_wildcard_rejected"));
    }
    normalized.sort();
    let unique = normalized.iter().collect::<BTreeSet<_>>();
    if unique.len() != normalized.len() {
        return Err(format!("execution_grant_{field}_contains_duplicate"));
    }
    Ok(normalized)
}

fn scope_fingerprint(
    project_id: &str,
    workflow_id: &str,
    allowed_work_item_types: &[String],
    allowed_role_ids: &[String],
    allowed_agent_ids: &[String],
    allowed_read_roots: &[String],
    allowed_write_roots: &[String],
    allowed_tools: &[String],
    allowed_checks: &[String],
    stop_conditions: &[String],
    max_worker_dispatches: Option<i64>,
    max_runtime_minutes: Option<i64>,
) -> String {
    crate::utils::hash::sha256_hex(
        &serde_json::json!({
            "project_id": project_id,
            "workflow_id": workflow_id,
            "allowed_work_item_types": allowed_work_item_types,
            "allowed_role_ids": allowed_role_ids,
            "allowed_agent_ids": allowed_agent_ids,
            "allowed_read_roots": allowed_read_roots,
            "allowed_write_roots": allowed_write_roots,
            "allowed_tools": allowed_tools,
            "allowed_checks": allowed_checks,
            "stop_conditions": stop_conditions,
            "max_worker_dispatches": max_worker_dispatches,
            "max_runtime_minutes": max_runtime_minutes,
        })
        .to_string(),
    )
}

fn grant_hash(grant: &ExecutionGrant) -> String {
    crate::utils::hash::sha256_hex(
        &serde_json::json!({
            "grant_id": grant.grant_id.0,
            "schema_version": grant.schema_version,
            "authorization_id": grant.authorization_id,
            "authorization_store_revision": grant.authorization_store_revision,
            "authorization_source_hash": grant.authorization_source_hash,
            "scope_fingerprint": grant.scope_fingerprint,
            "principal": grant.principal,
            "project_id": grant.project_id,
            "workflow_id": grant.workflow_id,
            "workflow_node_id": grant.workflow_node_id,
            "work_item_id": grant.work_item_id,
            "dispatch_id": grant.dispatch_id,
            "attempt_id": grant.attempt_id,
            "binding_id": grant.binding_id,
            "prepared_dispatch_id": grant.prepared_dispatch_id,
            "max_worker_dispatches": grant.max_worker_dispatches,
            "max_runtime_minutes": grant.max_runtime_minutes,
            "allowed_work_item_types": grant.allowed_work_item_types,
            "allowed_role_ids": grant.allowed_role_ids,
            "allowed_agent_ids": grant.allowed_agent_ids,
            "allowed_read_roots": grant.allowed_read_roots,
            "allowed_write_roots": grant.allowed_write_roots,
            "allowed_tools": grant.allowed_tools,
            "allowed_checks": grant.allowed_checks,
            "stop_conditions": grant.stop_conditions,
            "minted_at_ms": grant.minted_at_ms,
            "expires_at_ms": grant.expires_at_ms,
            "revoked_at_ms": grant.revoked_at_ms,
            "revoked_reason": grant.revoked_reason,
            "minted_by": grant.minted_by,
        })
        .to_string(),
    )
}

fn verify_integrity(grant: &ExecutionGrant) -> Result<(), String> {
    if grant.schema_version != GRANT_SCHEMA_VERSION {
        return Err("execution_grant_schema_mismatch".to_string());
    }
    if grant.authorization_store_revision < 0 || grant.expires_at_ms <= grant.minted_at_ms {
        return Err("execution_grant_clock_or_revision_invalid".to_string());
    }
    if !grant.grant_id.0.starts_with("grant:") || grant.grant_id.0.len() != 70 {
        return Err("execution_grant_id_invalid".to_string());
    }
    require_nonempty("authorization_id", &grant.authorization_id)?;
    require_nonempty("principal", &grant.principal)?;
    require_nonempty("project_id", &grant.project_id)?;
    require_nonempty("workflow_id", &grant.workflow_id)?;
    require_nonempty("minted_by", &grant.minted_by)?;
    let expected_scope = scope_fingerprint(
        &grant.project_id,
        &grant.workflow_id,
        &normalized_scope("allowed_work_item_types", &grant.allowed_work_item_types)?,
        &normalized_scope("allowed_role_ids", &grant.allowed_role_ids)?,
        &normalized_scope("allowed_agent_ids", &grant.allowed_agent_ids)?,
        &normalized_scope("allowed_read_roots", &grant.allowed_read_roots)?,
        &normalized_scope("allowed_write_roots", &grant.allowed_write_roots)?,
        &normalized_scope("allowed_tools", &grant.allowed_tools)?,
        &normalized_scope("allowed_checks", &grant.allowed_checks)?,
        &normalized_scope("stop_conditions", &grant.stop_conditions)?,
        grant.max_worker_dispatches,
        grant.max_runtime_minutes,
    );
    if grant.scope_fingerprint != expected_scope {
        return Err("execution_grant_scope_fingerprint_mismatch".to_string());
    }
    if grant.dispatch_id.is_some()
        || grant.attempt_id.is_some()
        || grant.workflow_node_id.is_some()
        || grant.work_item_id.is_some()
        || grant.binding_id.is_some()
    {
        for (field, value) in [
            ("dispatch_id", grant.dispatch_id.as_deref()),
            ("attempt_id", grant.attempt_id.as_deref()),
            ("workflow_node_id", grant.workflow_node_id.as_deref()),
            ("work_item_id", grant.work_item_id.as_deref()),
            ("binding_id", grant.binding_id.as_deref()),
            (
                "prepared_dispatch_id",
                grant.prepared_dispatch_id.as_deref(),
            ),
            (
                "authorization_source_hash",
                grant.authorization_source_hash.as_deref(),
            ),
        ] {
            require_nonempty(field, value.unwrap_or(""))?;
        }
        if grant.max_worker_dispatches.unwrap_or_default() <= 0
            || grant.max_runtime_minutes.unwrap_or_default() <= 0
        {
            return Err("execution_grant_dispatch_quota_invalid".to_string());
        }
    }
    Ok(())
}

fn random_identifier(prefix: &str) -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|error| format!("execution_grant_entropy_unavailable:{error}"))?;
    let mut value = String::with_capacity(prefix.len() + 1 + bytes.len() * 2);
    value.push_str(prefix);
    value.push(':');
    for byte in bytes {
        value.push_str(&format!("{byte:02x}"));
    }
    Ok(value)
}

fn unix_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> GrantMintInput {
        GrantMintInput {
            authorization_id: "auth-001".to_string(),
            authorization_revision: 7,
            scope_fingerprint: "caller-controlled-and-ignored".to_string(),
            principal: "thread:worker-001".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test:default".to_string(),
            allowed_work_item_types: vec!["task_package".to_string()],
            allowed_role_ids: vec!["worker".to_string()],
            allowed_agent_ids: vec!["thread:worker-001".to_string()],
            allowed_read_roots: vec!["/workspace/read".to_string()],
            allowed_write_roots: vec!["/workspace/write".to_string()],
            allowed_tools: vec!["bash".to_string(), "edit".to_string()],
            allowed_checks: vec!["cargo_test".to_string()],
            stop_conditions: vec!["user_rejected".to_string()],
            ttl_seconds: 3600,
            minted_by: "server".to_string(),
        }
    }

    #[test]
    fn mint_uses_real_clock_random_identifier_and_cryptographic_scope_hash() {
        let grant = mint_grant(&test_input()).expect("mint");
        assert!(grant.grant_id.0.starts_with("grant:"));
        assert_eq!(grant.grant_id.0.len(), 70);
        assert!(grant.minted_at_ms > 1_700_000_000_000);
        assert!(grant.expires_at_ms > grant.minted_at_ms);
        assert_eq!(grant.scope_fingerprint.len(), 64);
        assert_eq!(
            verify_grant(
                &grant,
                "project:test",
                "workflow:test:default",
                "worker",
                Some("thread:worker-001"),
                Some("bash"),
                Some("/workspace/write"),
            ),
            GrantVerification::Valid
        );
    }

    #[test]
    fn wildcard_and_duplicate_scope_are_rejected_before_mint() {
        let mut wildcard = test_input();
        wildcard.allowed_tools = vec!["*".to_string()];
        assert!(mint_grant(&wildcard)
            .unwrap_err()
            .contains("wildcard_rejected"));
        let mut duplicate = test_input();
        duplicate.allowed_role_ids.push("worker".to_string());
        assert!(mint_grant(&duplicate)
            .unwrap_err()
            .contains("contains_duplicate"));
    }

    #[test]
    fn tampered_or_expired_grant_is_rejected() {
        let mut tampered = mint_grant(&test_input()).expect("mint");
        tampered.allowed_tools.push("rm".to_string());
        assert!(matches!(
            verify_grant(
                &tampered,
                "project:test",
                "workflow:test:default",
                "worker",
                None,
                None,
                None
            ),
            GrantVerification::InvalidGrant { .. }
        ));
        let mut expired = mint_grant(&test_input()).expect("mint");
        let now = unix_timestamp_ms();
        expired.minted_at_ms = now - 2_000;
        expired.expires_at_ms = now - 1_000;
        expired.grant_hash = grant_hash(&expired);
        assert_eq!(
            verify_grant(
                &expired,
                "project:test",
                "workflow:test:default",
                "worker",
                None,
                None,
                None
            ),
            GrantVerification::Expired
        );
    }

    #[test]
    fn persisted_dispatch_grant_binds_every_server_context_field() {
        let source = ExecutionGrantAuthorizationSource {
            authorization_id: "plan-auth:approved".to_string(),
            authorization_store_revision: 12,
            authorization_source_hash: "fixture-source-hash".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test:default".to_string(),
            allowed_work_item_types: vec!["task_package".to_string()],
            allowed_role_ids: vec!["worker".to_string()],
            allowed_agent_ids: vec!["thread:worker-001".to_string()],
            allowed_read_roots: vec!["/workspace/read".to_string()],
            allowed_write_roots: vec!["/workspace/write".to_string()],
            allowed_tools: vec!["bash".to_string()],
            allowed_checks: vec!["cargo_test".to_string()],
            stop_conditions: vec!["user_rejected".to_string()],
            expires_at_ms: None,
            max_worker_dispatches: Some(1),
            max_runtime_minutes: Some(2),
        };
        let binding = ExecutionGrantBinding {
            dispatch_id: "dispatch:server-owned".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test:default".to_string(),
            workflow_node_id: "workflow:test:default:node:worker".to_string(),
            work_item_id: "wi:test".to_string(),
            binding_id: "binding:server-owned".to_string(),
            principal: "thread:worker-001".to_string(),
            prepared_dispatch_id: "prepared:fixture".to_string(),
        };
        let grant = mint_dispatch_grant(&source, &binding, 120).expect("mint dispatch grant");
        assert_ne!(grant.grant_id.0, binding.dispatch_id);
        assert_ne!(
            grant.attempt_id.as_deref(),
            Some(binding.dispatch_id.as_str())
        );
        assert_eq!(
            verify_dispatch_grant(
                &grant,
                &DispatchGrantVerificationContext {
                    project_id: "project:test",
                    workflow_id: "workflow:test:default",
                    workflow_node_id: "workflow:test:default:node:worker",
                    work_item_id: "wi:test",
                    dispatch_id: "dispatch:server-owned",
                    attempt_id: grant.attempt_id.as_deref().expect("attempt"),
                    binding_id: "binding:server-owned",
                    principal: "thread:worker-001",
                    actor_role: "worker",
                }
            ),
            GrantVerification::Valid
        );
        assert!(matches!(
            verify_dispatch_grant(
                &grant,
                &DispatchGrantVerificationContext {
                    project_id: "project:test",
                    workflow_id: "workflow:test:default",
                    workflow_node_id: "workflow:test:default:node:worker",
                    work_item_id: "wi:forged",
                    dispatch_id: "dispatch:server-owned",
                    attempt_id: grant.attempt_id.as_deref().expect("attempt"),
                    binding_id: "binding:server-owned",
                    principal: "thread:worker-001",
                    actor_role: "worker",
                }
            ),
            GrantVerification::BindingMismatch { .. }
        ));
        assert!(matches!(
            verify_dispatch_grant(
                &grant,
                &DispatchGrantVerificationContext {
                    project_id: "project:forged",
                    workflow_id: "workflow:test:default",
                    workflow_node_id: "workflow:test:default:node:worker",
                    work_item_id: "wi:test",
                    dispatch_id: "dispatch:server-owned",
                    attempt_id: grant.attempt_id.as_deref().expect("attempt"),
                    binding_id: "binding:server-owned",
                    principal: "thread:worker-001",
                    actor_role: "worker",
                }
            ),
            GrantVerification::ProjectMismatch { .. }
        ));
        assert!(matches!(
            verify_dispatch_grant(
                &grant,
                &DispatchGrantVerificationContext {
                    project_id: "project:test",
                    workflow_id: "workflow:forged",
                    workflow_node_id: "workflow:test:default:node:worker",
                    work_item_id: "wi:test",
                    dispatch_id: "dispatch:server-owned",
                    attempt_id: grant.attempt_id.as_deref().expect("attempt"),
                    binding_id: "binding:server-owned",
                    principal: "thread:worker-001",
                    actor_role: "worker",
                }
            ),
            GrantVerification::WorkflowMismatch { .. }
        ));
        assert!(matches!(
            verify_dispatch_grant(
                &grant,
                &DispatchGrantVerificationContext {
                    project_id: "project:test",
                    workflow_id: "workflow:test:default",
                    workflow_node_id: "workflow:test:default:node:worker",
                    work_item_id: "wi:test",
                    dispatch_id: "dispatch:server-owned",
                    attempt_id: grant.attempt_id.as_deref().expect("attempt"),
                    binding_id: "binding:server-owned",
                    principal: "thread:worker-001",
                    actor_role: "forged-role",
                }
            ),
            GrantVerification::RoleNotAllowed { .. }
        ));
    }

    #[test]
    fn revocation_changes_integrity_material_and_fails_closed() {
        let mut grant = mint_grant(&test_input()).expect("mint");
        revoke_grant(&mut grant, "operator revoked");
        assert!(matches!(
            verify_grant(
                &grant,
                "project:test",
                "workflow:test:default",
                "worker",
                None,
                None,
                None
            ),
            GrantVerification::Revoked { .. }
        ));
    }
}
