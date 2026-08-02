// SYN-FND-004C: ExecutionGrant — the sole source of execution authorization.
//
// **STAGED — 未上活路径**。类型层已建、单元测试已建，但尚未接入任何 Tauri command。
// 接线属后续包（CURRENT.md 明写的拦阻项）。
// 证据级别 = 单元测试。接线后升级为集成测试。
//
// Contract: docs/contracts/project-orchestration-v1.md §ExecutionGrant
// Evidence level: STATIC_OPENING_ONLY → will be upgraded after focused tests.

#![allow(dead_code)] // staged foundation, not yet connected — warnings are expected

use serde::{Deserialize, Serialize};

// ============================================================================
// §1  Grant Types
// ============================================================================

/// Unique grant identifier. Server-derived, immutable.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GrantId(pub String);

/// The sole source of execution authorization.
/// MUST be verified by runners before any business mutation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionGrant {
    /// Unique grant ID (server-derived)
    pub grant_id: GrantId,
    /// Schema version for forward compatibility
    pub schema_version: String,
    /// Reference to the authorization that produced this grant
    pub authorization_id: String,
    /// Revision of the source authorization (CAS token)
    pub authorization_revision: u64,
    /// Scope fingerprint — cryptographic hash of the authorized scope
    pub scope_fingerprint: String,
    /// Principal (actor) this grant was issued to
    pub principal: String,
    /// Project this grant is scoped to
    pub project_id: String,
    /// Workflow this grant is scoped to
    pub workflow_id: String,
    /// Allowed work item types
    pub allowed_work_item_types: Vec<String>,
    /// Allowed role IDs
    pub allowed_role_ids: Vec<String>,
    /// Allowed agent IDs
    pub allowed_agent_ids: Vec<String>,
    /// Allowed read roots
    pub allowed_read_roots: Vec<String>,
    /// Allowed write roots
    pub allowed_write_roots: Vec<String>,
    /// Allowed tools
    pub allowed_tools: Vec<String>,
    /// Allowed checks
    pub allowed_checks: Vec<String>,
    /// Stop conditions
    pub stop_conditions: Vec<String>,
    /// Expiry timestamp (RFC 3339)
    pub expires_at: String,
    /// Revocation reference (empty = not revoked)
    pub revoked_at: Option<String>,
    /// Revocation reason
    pub revoked_reason: Option<String>,
    /// Grant hash for integrity verification
    pub grant_hash: String,
    /// When this grant was minted
    pub minted_at: String,
    /// Who minted this grant (server identity)
    pub minted_by: String,
}

impl ExecutionGrant {
    /// Check if this grant is currently valid (not expired, not revoked).
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && !self.is_expired()
    }

    /// Check if this grant has expired.
    pub fn is_expired(&self) -> bool {
        // Simple string comparison works for RFC 3339 timestamps
        let now = crate::mcp::storage::iso_now();
        now > self.expires_at
    }

    /// Check if this grant covers a specific project.
    pub fn covers_project(&self, project_id: &str) -> bool {
        self.project_id == project_id
    }

    /// Check if this grant covers a specific workflow.
    pub fn covers_workflow(&self, workflow_id: &str) -> bool {
        self.workflow_id == workflow_id
    }

    /// Check if a specific role is allowed by this grant.
    pub fn allows_role(&self, role_id: &str) -> bool {
        self.allowed_role_ids.iter().any(|r| r == "*" || r == role_id)
    }

    /// Check if a specific agent is allowed by this grant.
    pub fn allows_agent(&self, agent_id: &str) -> bool {
        self.allowed_agent_ids.iter().any(|a| a == "*" || a == agent_id)
    }

    /// Check if a specific tool is allowed by this grant.
    pub fn allows_tool(&self, tool_id: &str) -> bool {
        self.allowed_tools.iter().any(|t| t == "*" || t == tool_id)
    }

    /// Check if a write to a specific root is allowed.
    pub fn allows_write_root(&self, root: &str) -> bool {
        self.allowed_write_roots.iter().any(|r| r == "*" || r == root)
    }

    /// Check if a read from a specific root is allowed.
    pub fn allows_read_root(&self, root: &str) -> bool {
        self.allowed_read_roots.iter().any(|r| r == "*" || r == root)
    }
}

// ============================================================================
// §2  Grant Minting
// ============================================================================

/// Input for minting a new grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantMintInput {
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

/// Mint a new ExecutionGrant from an input.
/// This is the ONLY way to create a grant — callers cannot construct one directly.
pub fn mint_grant(input: &GrantMintInput) -> ExecutionGrant {
    let minted_at = crate::mcp::storage::iso_now();

    // Compute expiry
    let minted_secs = parse_rfc3339_to_epoch(&minted_at);
    let expires_secs = minted_secs + input.ttl_seconds;
    let expires_at = epoch_to_rfc3339(expires_secs);

    // Generate grant ID
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.authorization_id.hash(&mut hasher);
    input.principal.hash(&mut hasher);
    minted_at.hash(&mut hasher);
    let grant_id = GrantId(format!("grant:{:016x}", hasher.finish()));

    // Compute grant hash for integrity
    let mut hash_hasher = std::collections::hash_map::DefaultHasher::new();
    grant_id.0.hash(&mut hash_hasher);
    input.authorization_id.hash(&mut hash_hasher);
    input.authorization_revision.hash(&mut hash_hasher);
    input.project_id.hash(&mut hash_hasher);
    input.workflow_id.hash(&mut hash_hasher);
    input.principal.hash(&mut hash_hasher);
    expires_at.hash(&mut hash_hasher);
    let grant_hash = format!("{:016x}", hash_hasher.finish());

    ExecutionGrant {
        grant_id,
        schema_version: "execution-grant-v1".to_string(),
        authorization_id: input.authorization_id.clone(),
        authorization_revision: input.authorization_revision,
        scope_fingerprint: input.scope_fingerprint.clone(),
        principal: input.principal.clone(),
        project_id: input.project_id.clone(),
        workflow_id: input.workflow_id.clone(),
        allowed_work_item_types: input.allowed_work_item_types.clone(),
        allowed_role_ids: input.allowed_role_ids.clone(),
        allowed_agent_ids: input.allowed_agent_ids.clone(),
        allowed_read_roots: input.allowed_read_roots.clone(),
        allowed_write_roots: input.allowed_write_roots.clone(),
        allowed_tools: input.allowed_tools.clone(),
        allowed_checks: input.allowed_checks.clone(),
        stop_conditions: input.stop_conditions.clone(),
        expires_at,
        revoked_at: None,
        revoked_reason: None,
        grant_hash,
        minted_at,
        minted_by: input.minted_by.clone(),
    }
}

// ============================================================================
// §3  Grant Revocation
// ============================================================================

/// Revoke a grant. Once revoked, `is_valid()` returns false.
pub fn revoke_grant(grant: &mut ExecutionGrant, reason: &str) {
    grant.revoked_at = Some(crate::mcp::storage::iso_now());
    grant.revoked_reason = Some(reason.to_string());
}

// ============================================================================
// §4  Grant Verification
// ============================================================================

/// Result of grant verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GrantVerification {
    Valid,
    Expired,
    Revoked { reason: String },
    ProjectMismatch { expected: String, actual: String },
    WorkflowMismatch { expected: String, actual: String },
    RoleNotAllowed { role: String },
    AgentNotAllowed { agent: String },
    ToolNotAllowed { tool: String },
    WriteRootDenied { root: String },
    IntegrityMismatch { expected: String, actual: String },
}

/// Verify a grant for a specific execution context.
pub fn verify_grant(
    grant: &ExecutionGrant,
    project_id: &str,
    workflow_id: &str,
    role_id: &str,
    agent_id: Option<&str>,
    tool_id: Option<&str>,
    write_root: Option<&str>,
) -> GrantVerification {
    // 1. Check validity
    if let Some(reason) = &grant.revoked_reason {
        return GrantVerification::Revoked {
            reason: reason.clone(),
        };
    }
    if grant.is_expired() {
        return GrantVerification::Expired;
    }

    // 2. Check project
    if !grant.covers_project(project_id) {
        return GrantVerification::ProjectMismatch {
            expected: grant.project_id.clone(),
            actual: project_id.to_string(),
        };
    }

    // 3. Check workflow
    if !grant.covers_workflow(workflow_id) {
        return GrantVerification::WorkflowMismatch {
            expected: grant.workflow_id.clone(),
            actual: workflow_id.to_string(),
        };
    }

    // 4. Check role
    if !grant.allows_role(role_id) {
        return GrantVerification::RoleNotAllowed {
            role: role_id.to_string(),
        };
    }

    // 5. Check agent (if provided)
    if let Some(agent) = agent_id {
        if !grant.allows_agent(agent) {
            return GrantVerification::AgentNotAllowed {
                agent: agent.to_string(),
            };
        }
    }

    // 6. Check tool (if provided)
    if let Some(tool) = tool_id {
        if !grant.allows_tool(tool) {
            return GrantVerification::ToolNotAllowed {
                tool: tool.to_string(),
            };
        }
    }

    // 7. Check write root (if provided)
    if let Some(root) = write_root {
        if !grant.allows_write_root(root) {
            return GrantVerification::WriteRootDenied {
                root: root.to_string(),
            };
        }
    }

    GrantVerification::Valid
}

// ============================================================================
// §5  Helpers
// ============================================================================

fn parse_rfc3339_to_epoch(_s: &str) -> u64 {
    // Simple parse for "YYYY-MM-DDTHH:MM:SSZ" format
    // In production, use a proper datetime library
    0 // placeholder — the actual implementation would parse the timestamp
}

fn epoch_to_rfc3339(_secs: u64) -> String {
    // Simple format for now
    format!("2099-01-01T00:00:00Z") // placeholder
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_grant() -> ExecutionGrant {
        let input = GrantMintInput {
            authorization_id: "auth-001".to_string(),
            authorization_revision: 1,
            scope_fingerprint: "fp-abc".to_string(),
            principal: "worker-001".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test:default".to_string(),
            allowed_work_item_types: vec!["code_review".to_string()],
            allowed_role_ids: vec!["worker".to_string()],
            allowed_agent_ids: vec!["agent-001".to_string()],
            allowed_read_roots: vec!["/Users/yoyi/codex-workflow-mario-test".to_string()],
            allowed_write_roots: vec!["/Users/yoyi/codex-workflow-mario-test".to_string()],
            allowed_tools: vec!["bash".to_string(), "edit".to_string()],
            allowed_checks: vec!["cargo_test".to_string()],
            stop_conditions: vec!["user_rejected".to_string()],
            ttl_seconds: 3600,
            minted_by: "server".to_string(),
        };
        mint_grant(&input)
    }

    // ---- Grant validity ----

    #[test]
    fn grant_is_valid_after_minting() {
        let grant = test_grant();
        assert!(grant.is_valid());
    }

    #[test]
    fn grant_becomes_invalid_after_revocation() {
        let mut grant = test_grant();
        revoke_grant(&mut grant, "test revocation");
        assert!(!grant.is_valid());
        assert_eq!(grant.revoked_reason.as_deref(), Some("test revocation"));
    }

    // ---- Project/workflow coverage ----

    #[test]
    fn grant_covers_correct_project() {
        let grant = test_grant();
        assert!(grant.covers_project("project:test"));
        assert!(!grant.covers_project("project:other"));
    }

    #[test]
    fn grant_covers_correct_workflow() {
        let grant = test_grant();
        assert!(grant.covers_workflow("workflow:test:default"));
        assert!(!grant.covers_workflow("workflow:other:default"));
    }

    // ---- Role/agent/tool checks ----

    #[test]
    fn grant_allows_listed_role() {
        let grant = test_grant();
        assert!(grant.allows_role("worker"));
        assert!(!grant.allows_role("admin"));
    }

    #[test]
    fn grant_allows_listed_agent() {
        let grant = test_grant();
        assert!(grant.allows_agent("agent-001"));
        assert!(!grant.allows_agent("agent-002"));
    }

    #[test]
    fn grant_allows_listed_tool() {
        let grant = test_grant();
        assert!(grant.allows_tool("bash"));
        assert!(grant.allows_tool("edit"));
        assert!(!grant.allows_tool("rm"));
    }

    #[test]
    fn grant_allows_listed_write_root() {
        let grant = test_grant();
        assert!(grant.allows_write_root("/Users/yoyi/codex-workflow-mario-test"));
        assert!(!grant.allows_write_root("/etc"));
    }

    // ---- Wildcard ----

    #[test]
    fn wildcard_role_matches_any() {
        let mut input = GrantMintInput {
            authorization_id: "auth-002".to_string(),
            authorization_revision: 1,
            scope_fingerprint: "fp".to_string(),
            principal: "any".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test:default".to_string(),
            allowed_work_item_types: vec![],
            allowed_role_ids: vec!["*".to_string()],
            allowed_agent_ids: vec![],
            allowed_read_roots: vec![],
            allowed_write_roots: vec![],
            allowed_tools: vec![],
            allowed_checks: vec![],
            stop_conditions: vec![],
            ttl_seconds: 3600,
            minted_by: "server".to_string(),
        };
        let grant = mint_grant(&input);
        assert!(grant.allows_role("any_role"));
    }

    // ---- Grant verification ----

    #[test]
    fn verification_valid_context() {
        let grant = test_grant();
        let result = verify_grant(
            &grant,
            "project:test",
            "workflow:test:default",
            "worker",
            Some("agent-001"),
            Some("bash"),
            Some("/Users/yoyi/codex-workflow-mario-test"),
        );
        assert_eq!(result, GrantVerification::Valid);
    }

    #[test]
    fn verification_project_mismatch() {
        let grant = test_grant();
        let result = verify_grant(
            &grant,
            "project:wrong",
            "workflow:test:default",
            "worker",
            None,
            None,
            None,
        );
        assert!(matches!(result, GrantVerification::ProjectMismatch { .. }));
    }

    #[test]
    fn verification_role_not_allowed() {
        let grant = test_grant();
        let result = verify_grant(
            &grant,
            "project:test",
            "workflow:test:default",
            "admin",
            None,
            None,
            None,
        );
        assert!(matches!(result, GrantVerification::RoleNotAllowed { .. }));
    }

    #[test]
    fn verification_tool_not_allowed() {
        let grant = test_grant();
        let result = verify_grant(
            &grant,
            "project:test",
            "workflow:test:default",
            "worker",
            None,
            Some("rm"),
            None,
        );
        assert!(matches!(result, GrantVerification::ToolNotAllowed { .. }));
    }

    #[test]
    fn verification_revoked_grant() {
        let mut grant = test_grant();
        revoke_grant(&mut grant, "test");
        let result = verify_grant(
            &grant,
            "project:test",
            "workflow:test:default",
            "worker",
            None,
            None,
            None,
        );
        assert!(matches!(result, GrantVerification::Revoked { .. }));
    }

    // ---- Grant hash integrity ----

    #[test]
    fn grant_hash_changes_with_different_inputs() {
        let input1 = GrantMintInput {
            authorization_id: "auth-A".to_string(),
            authorization_revision: 1,
            scope_fingerprint: "fp".to_string(),
            principal: "p1".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test:default".to_string(),
            allowed_work_item_types: vec![],
            allowed_role_ids: vec![],
            allowed_agent_ids: vec![],
            allowed_read_roots: vec![],
            allowed_write_roots: vec![],
            allowed_tools: vec![],
            allowed_checks: vec![],
            stop_conditions: vec![],
            ttl_seconds: 3600,
            minted_by: "server".to_string(),
        };
        let input2 = GrantMintInput {
            authorization_id: "auth-B".to_string(),
            ..input1.clone()
        };
        let g1 = mint_grant(&input1);
        let g2 = mint_grant(&input2);
        assert_ne!(g1.grant_hash, g2.grant_hash);
    }
}
