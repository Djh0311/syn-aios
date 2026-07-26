//! Server-owned MCP capability registry.
//!
//! `tools/list` and `tools/call` must both ask this module for authorization.
//! The client never supplies a profile, role, or expandable capability list: the
//! host establishes the [`CapabilityAccess`] from durable session state.

use std::collections::BTreeSet;
use std::fmt;

pub(crate) const PROFILE_AGENT_CODEX_WORKSPACE_WRITE: &str = "agent-codex-workspace-write";
pub(crate) const PROFILE_SUPERVISOR_READ_ONLY: &str = "supervisor-read-only";

// These two profile identifiers are compatibility-only host mappings.  They
// preserve the existing supervisor MCP surfaces while keeping the new
// conversation profile independently and exactly allowlisted.
pub(crate) const PROFILE_SUPERVISOR_ORCHESTRATOR_LEGACY: &str = "supervisor-orchestrator-legacy";
pub(crate) const PROFILE_SUPERVISOR_RESIDENT_LEGACY: &str = "supervisor-resident-legacy";

pub(crate) const ROLE_PROJECT_SUPERVISOR: &str = "project_supervisor";
pub(crate) const ROLE_SUPERVISOR_ORCHESTRATOR: &str = "supervisor_orchestrator";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapabilityHandler {
    ReadWorkerReport,
    WaitForWorker,
    ReadKeyFile,
    SubmitProposal,
    KnowledgeSearch,
    KnowledgeRead,
    KnowledgeOpen,
    KnowledgeCite,
}

impl CapabilityHandler {
    pub(crate) const fn tool_name(self) -> &'static str {
        match self {
            Self::ReadWorkerReport => "read_worker_report",
            Self::WaitForWorker => "wait_for_worker",
            Self::ReadKeyFile => "read_key_file",
            Self::SubmitProposal => "submit_proposal",
            Self::KnowledgeSearch => "knowledge_search",
            Self::KnowledgeRead => "knowledge_read",
            Self::KnowledgeOpen => "knowledge_open",
            Self::KnowledgeCite => "knowledge_cite",
        }
    }
}

/// The registry owns schema selection too.  The actual JSON schema stays with
/// the existing handler module so compatibility callers keep their current
/// shape, while list and call still share this one capability decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapabilitySchema {
    WorkerId,
    ReadKeyFile,
    SubmitProposal,
    KnowledgeSearch,
    KnowledgeRead,
    KnowledgeOpen,
    KnowledgeCite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapabilityDefinition {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) schema: CapabilitySchema,
    pub(crate) handler: CapabilityHandler,
    pub(crate) allowed_profiles: &'static [&'static str],
    pub(crate) allowed_roles: &'static [&'static str],
    pub(crate) audit_event_type: &'static str,
    pub(crate) denied_message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CapabilityAccess<'a> {
    pub(crate) profile_id: &'a str,
    pub(crate) role: &'a str,
}

impl<'a> CapabilityAccess<'a> {
    pub(crate) const fn new(profile_id: &'a str, role: &'a str) -> Self {
        Self { profile_id, role }
    }
}

const LEGACY_READ_ONLY_TOOL_PROFILES: &[&str] = &[
    PROFILE_SUPERVISOR_ORCHESTRATOR_LEGACY,
    PROFILE_SUPERVISOR_RESIDENT_LEGACY,
];
const SUBMIT_PROPOSAL_PROFILES: &[&str] = &[
    PROFILE_SUPERVISOR_READ_ONLY,
    PROFILE_SUPERVISOR_RESIDENT_LEGACY,
];
const SUPERVISOR_READ_ONLY_KNOWLEDGE_PROFILES: &[&str] = &[PROFILE_SUPERVISOR_READ_ONLY];
const ORCHESTRATOR_OR_PROJECT_SUPERVISOR_ROLES: &[&str] =
    &[ROLE_SUPERVISOR_ORCHESTRATOR, ROLE_PROJECT_SUPERVISOR];
const PROJECT_SUPERVISOR_ONLY: &[&str] = &[ROLE_PROJECT_SUPERVISOR];

const REGISTRY: &[CapabilityDefinition] = &[
    CapabilityDefinition {
        name: "read_worker_report",
        description: "只读投影 worker 结构化口供",
        schema: CapabilitySchema::WorkerId,
        handler: CapabilityHandler::ReadWorkerReport,
        allowed_profiles: LEGACY_READ_ONLY_TOOL_PROFILES,
        allowed_roles: ORCHESTRATOR_OR_PROJECT_SUPERVISOR_ROLES,
        audit_event_type: "supervisor_read_worker_report",
        denied_message: "当前会话未获授权读取 worker 报告。",
    },
    CapabilityDefinition {
        name: "wait_for_worker",
        description: "读取 worker 当前状态，不管理或终止进程",
        schema: CapabilitySchema::WorkerId,
        handler: CapabilityHandler::WaitForWorker,
        allowed_profiles: LEGACY_READ_ONLY_TOOL_PROFILES,
        allowed_roles: ORCHESTRATOR_OR_PROJECT_SUPERVISOR_ROLES,
        audit_event_type: "supervisor_wait_for_worker",
        denied_message: "当前会话未获授权读取 worker 状态。",
    },
    CapabilityDefinition {
        name: "read_key_file",
        description: "在授权允许读取根内读取关键文本文件",
        schema: CapabilitySchema::ReadKeyFile,
        handler: CapabilityHandler::ReadKeyFile,
        allowed_profiles: LEGACY_READ_ONLY_TOOL_PROFILES,
        allowed_roles: ORCHESTRATOR_OR_PROJECT_SUPERVISOR_ROLES,
        audit_event_type: "supervisor_read_key_file",
        denied_message: "当前会话未获授权读取关键文件。",
    },
    CapabilityDefinition {
        name: "submit_proposal",
        description: "将已达成共识的终版方案落为待用户确认卡；不会自动推进工作流",
        schema: CapabilitySchema::SubmitProposal,
        handler: CapabilityHandler::SubmitProposal,
        allowed_profiles: SUBMIT_PROPOSAL_PROFILES,
        allowed_roles: PROJECT_SUPERVISOR_ONLY,
        audit_event_type: "supervisor_submit_proposal",
        denied_message: "当前会话未获授权生成待用户确认方案卡。",
    },
    CapabilityDefinition {
        name: "knowledge_search",
        description: "只在固定 Syn Markdown vault 内搜索受限文本",
        schema: CapabilitySchema::KnowledgeSearch,
        handler: CapabilityHandler::KnowledgeSearch,
        allowed_profiles: SUPERVISOR_READ_ONLY_KNOWLEDGE_PROFILES,
        allowed_roles: PROJECT_SUPERVISOR_ONLY,
        audit_event_type: "supervisor_knowledge_search",
        denied_message: "当前会话未获授权搜索 Syn 知识库。",
    },
    CapabilityDefinition {
        name: "knowledge_read",
        description: "按精确 relative_path 只读固定 Syn 原生工作区 Markdown 笔记",
        schema: CapabilitySchema::KnowledgeRead,
        handler: CapabilityHandler::KnowledgeRead,
        allowed_profiles: SUPERVISOR_READ_ONLY_KNOWLEDGE_PROFILES,
        allowed_roles: PROJECT_SUPERVISOR_ONLY,
        audit_event_type: "supervisor_knowledge_read",
        denied_message: "当前会话未获授权读取 Syn 知识库。",
    },
    CapabilityDefinition {
        name: "knowledge_open",
        description:
            "为固定 Syn 原生工作区笔记返回受限 native-view intent；不打开外部应用、不写入知识",
        schema: CapabilitySchema::KnowledgeOpen,
        handler: CapabilityHandler::KnowledgeOpen,
        allowed_profiles: SUPERVISOR_READ_ONLY_KNOWLEDGE_PROFILES,
        allowed_roles: PROJECT_SUPERVISOR_ONLY,
        audit_event_type: "supervisor_knowledge_open",
        denied_message: "当前会话未获授权打开 Syn 知识库笔记。",
    },
    CapabilityDefinition {
        name: "knowledge_cite",
        description: "为固定 Syn 原生工作区 Markdown 笔记生成只读结构化引用",
        schema: CapabilitySchema::KnowledgeCite,
        handler: CapabilityHandler::KnowledgeCite,
        allowed_profiles: SUPERVISOR_READ_ONLY_KNOWLEDGE_PROFILES,
        allowed_roles: PROJECT_SUPERVISOR_ONLY,
        audit_event_type: "supervisor_knowledge_cite",
        denied_message: "当前会话未获授权引用 Syn 知识库。",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CapabilityAuthorizationError {
    InvalidAccessContext,
    ProfileRoleNotRegistered {
        profile_id: String,
        role: String,
    },
    InvalidCapabilityName(String),
    UnknownCapability(String),
    CapabilityNotAllowed {
        name: String,
        profile_id: String,
        role: String,
    },
    EmptyCapabilitySet,
    DuplicateCapability(String),
    CapabilitySetNotExact {
        expected: Vec<String>,
        actual: Vec<String>,
    },
}

impl fmt::Display for CapabilityAuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAccessContext => {
                write!(formatter, "MCP capability 上下文不完整，已拒绝。")
            }
            Self::ProfileRoleNotRegistered { .. } => {
                write!(
                    formatter,
                    "MCP profile/role 未在服务端 capability registry 登记，已拒绝。"
                )
            }
            Self::InvalidCapabilityName(_) => {
                write!(formatter, "MCP capability 名称不是精确注册名，已拒绝。")
            }
            Self::UnknownCapability(_) => write!(formatter, "MCP capability 未注册，已拒绝。"),
            Self::CapabilityNotAllowed { .. } => {
                write!(formatter, "当前 MCP profile/role 未获该 capability 授权。")
            }
            Self::EmptyCapabilitySet => {
                write!(formatter, "MCP capability 集合不能为空或默认放开，已拒绝。")
            }
            Self::DuplicateCapability(_) => {
                write!(formatter, "MCP capability 集合含重复项，已拒绝。")
            }
            Self::CapabilitySetNotExact { .. } => {
                write!(
                    formatter,
                    "MCP capability 集合与宿主冻结 allowlist 不一致，已拒绝。"
                )
            }
        }
    }
}

impl std::error::Error for CapabilityAuthorizationError {}

pub(crate) fn registry() -> &'static [CapabilityDefinition] {
    REGISTRY
}

/// Return the capabilities visible to a host-established profile and role.
/// An unknown profile/role is an error rather than an empty, implicitly-open
/// result, so callers cannot accidentally turn it into a fallback allowlist.
pub(crate) fn list_allowed_capabilities(
    access: CapabilityAccess<'_>,
) -> Result<Vec<&'static CapabilityDefinition>, CapabilityAuthorizationError> {
    validate_access_context(access)?;
    let allowed = REGISTRY
        .iter()
        .filter(|definition| definition_allows(definition, access))
        .collect::<Vec<_>>();
    if allowed.is_empty() {
        return Err(CapabilityAuthorizationError::ProfileRoleNotRegistered {
            profile_id: access.profile_id.to_string(),
            role: access.role.to_string(),
        });
    }
    Ok(allowed)
}

/// Authorize one exact registered tool name.  There are deliberately no
/// aliases, case folding, wildcard matching, or default capabilities.
pub(crate) fn authorize(
    access: CapabilityAccess<'_>,
    requested_name: &str,
) -> Result<&'static CapabilityDefinition, CapabilityAuthorizationError> {
    validate_access_context(access)?;
    validate_exact_name(requested_name)?;
    let definition = REGISTRY
        .iter()
        .find(|definition| definition.name == requested_name)
        .ok_or_else(|| {
            CapabilityAuthorizationError::UnknownCapability(requested_name.to_string())
        })?;
    if !definition_allows(definition, access) {
        return Err(CapabilityAuthorizationError::CapabilityNotAllowed {
            name: requested_name.to_string(),
            profile_id: access.profile_id.to_string(),
            role: access.role.to_string(),
        });
    }
    Ok(definition)
}

/// Validate a persisted, host-frozen capability set.  A valid set must be
/// non-empty, unique, contain only exact registered names, and equal the
/// complete server allowlist for this access context.
pub(crate) fn validate_exact_capability_set(
    access: CapabilityAccess<'_>,
    requested_names: &[String],
) -> Result<Vec<&'static CapabilityDefinition>, CapabilityAuthorizationError> {
    let expected = list_allowed_capabilities(access)?;
    if requested_names.is_empty() {
        return Err(CapabilityAuthorizationError::EmptyCapabilitySet);
    }

    let mut seen = BTreeSet::new();
    for name in requested_names {
        if !seen.insert(name.as_str()) {
            return Err(CapabilityAuthorizationError::DuplicateCapability(
                name.clone(),
            ));
        }
        authorize(access, name)?;
    }

    let expected_names = expected
        .iter()
        .map(|definition| definition.name.to_string())
        .collect::<BTreeSet<_>>();
    let actual_names = requested_names.iter().cloned().collect::<BTreeSet<_>>();
    if actual_names != expected_names {
        return Err(CapabilityAuthorizationError::CapabilitySetNotExact {
            expected: expected_names.into_iter().collect(),
            actual: actual_names.into_iter().collect(),
        });
    }
    Ok(expected)
}

pub(crate) fn frozen_capability_names(
    access: CapabilityAccess<'_>,
) -> Result<Vec<String>, CapabilityAuthorizationError> {
    list_allowed_capabilities(access).map(|definitions| {
        definitions
            .into_iter()
            .map(|definition| definition.name.to_string())
            .collect()
    })
}

fn validate_access_context(
    access: CapabilityAccess<'_>,
) -> Result<(), CapabilityAuthorizationError> {
    if access.profile_id.trim().is_empty()
        || access.role.trim().is_empty()
        || access.profile_id != access.profile_id.trim()
        || access.role != access.role.trim()
    {
        return Err(CapabilityAuthorizationError::InvalidAccessContext);
    }
    Ok(())
}

fn validate_exact_name(requested_name: &str) -> Result<(), CapabilityAuthorizationError> {
    if requested_name.trim().is_empty()
        || requested_name != requested_name.trim()
        || requested_name.contains('*')
    {
        return Err(CapabilityAuthorizationError::InvalidCapabilityName(
            requested_name.to_string(),
        ));
    }
    Ok(())
}

fn definition_allows(definition: &CapabilityDefinition, access: CapabilityAccess<'_>) -> bool {
    definition
        .allowed_profiles
        .iter()
        .any(|profile| *profile == access.profile_id)
        && definition
            .allowed_roles
            .iter()
            .any(|role| *role == access.role)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supervisor_read_only_access() -> CapabilityAccess<'static> {
        CapabilityAccess::new(PROFILE_SUPERVISOR_READ_ONLY, ROLE_PROJECT_SUPERVISOR)
    }

    #[test]
    fn supervisor_read_only_lists_the_five_exact_host_capabilities() {
        let access = supervisor_read_only_access();
        let names = list_allowed_capabilities(access)
            .unwrap()
            .into_iter()
            .map(|definition| definition.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "submit_proposal",
                "knowledge_search",
                "knowledge_read",
                "knowledge_open",
                "knowledge_cite"
            ]
        );
        assert_eq!(
            authorize(access, "submit_proposal").unwrap().handler,
            CapabilityHandler::SubmitProposal
        );
        assert_eq!(
            authorize(access, "knowledge_search").unwrap().handler,
            CapabilityHandler::KnowledgeSearch
        );
        assert_eq!(
            authorize(access, "knowledge_read").unwrap().handler,
            CapabilityHandler::KnowledgeRead
        );
        assert_eq!(
            authorize(access, "knowledge_open").unwrap().handler,
            CapabilityHandler::KnowledgeOpen
        );
        assert_eq!(
            authorize(access, "knowledge_cite").unwrap().handler,
            CapabilityHandler::KnowledgeCite
        );

        for denied in [
            "read_worker_report",
            "dispatch_worker",
            "SUBMIT_PROPOSAL",
            "KNOWLEDGE_READ",
            "knowledge_read ",
            "knowledge_write",
            "canvas_write",
            "attachment_write",
            "submit_proposal ",
            "*",
        ] {
            assert!(
                authorize(access, denied).is_err(),
                "{denied} must fail closed"
            );
        }
    }

    #[test]
    fn exact_capability_set_rejects_empty_duplicate_and_expansion() {
        let access = supervisor_read_only_access();
        let exact = frozen_capability_names(access).unwrap();
        assert!(matches!(
            validate_exact_capability_set(access, &[]),
            Err(CapabilityAuthorizationError::EmptyCapabilitySet)
        ));
        assert!(matches!(
            validate_exact_capability_set(
                access,
                &["submit_proposal".to_string(), "submit_proposal".to_string()]
            ),
            Err(CapabilityAuthorizationError::DuplicateCapability(_))
        ));
        assert!(matches!(
            validate_exact_capability_set(
                access,
                &["submit_proposal".to_string(), "knowledge_write".to_string()]
            ),
            Err(CapabilityAuthorizationError::UnknownCapability(_))
        ));
        assert!(matches!(
            validate_exact_capability_set(access, &["submit_proposal".to_string()]),
            Err(CapabilityAuthorizationError::CapabilitySetNotExact { .. })
        ));
        assert!(validate_exact_capability_set(access, &exact).is_ok());
    }

    #[test]
    fn legacy_resident_is_explicit_not_a_fallback_for_new_profile() {
        let legacy =
            CapabilityAccess::new(PROFILE_SUPERVISOR_RESIDENT_LEGACY, ROLE_PROJECT_SUPERVISOR);
        let names = list_allowed_capabilities(legacy)
            .unwrap()
            .into_iter()
            .map(|definition| definition.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "read_worker_report",
                "wait_for_worker",
                "read_key_file",
                "submit_proposal"
            ]
        );
        assert!(list_allowed_capabilities(CapabilityAccess::new(
            PROFILE_AGENT_CODEX_WORKSPACE_WRITE,
            ROLE_PROJECT_SUPERVISOR,
        ))
        .is_err());
    }

    #[test]
    fn registry_names_and_handlers_are_unique() {
        let mut names = BTreeSet::new();
        let mut handlers = BTreeSet::new();
        for definition in registry() {
            assert!(names.insert(definition.name));
            assert!(handlers.insert(definition.handler.tool_name()));
            assert_eq!(definition.name, definition.handler.tool_name());
        }
    }
}
