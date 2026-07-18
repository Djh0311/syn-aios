use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) const SUPERVISOR_ACTION_PROPOSAL_SCHEMA_V1: &str = "supervisor_action_proposal.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SupervisorActionProposalV1 {
    pub(crate) schema_version: String,
    #[serde(flatten)]
    pub(crate) action: SupervisorActionKind,
    pub(crate) reason: String,
    pub(crate) expected_result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SupervisorActionKind {
    DispatchWorker { target: SupervisorActionTarget },
    InspectWorker { worker_id: String },
    FollowUpWorker { worker_id: String, prompt: String },
    WaitWorker { worker_id: String },
    Finalize { verdict: SupervisorFinalizeVerdict },
    ReportUser { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SupervisorActionTarget {
    pub(crate) node_id: String,
    pub(crate) work_item_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SupervisorFinalizeVerdict {
    Pass,
    NeedsRework,
    Blocked,
}

impl SupervisorActionKind {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::DispatchWorker { .. } => "dispatch_worker",
            Self::InspectWorker { .. } => "inspect_worker",
            Self::FollowUpWorker { .. } => "follow_up_worker",
            Self::WaitWorker { .. } => "wait_worker",
            Self::Finalize { .. } => "finalize",
            Self::ReportUser { .. } => "report_user",
        }
    }
}

impl SupervisorFinalizeVerdict {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::NeedsRework => "needs_rework",
            Self::Blocked => "blocked",
        }
    }
}

pub(crate) fn parse_supervisor_action_proposal(
    last_message: &str,
) -> Result<SupervisorActionProposalV1, String> {
    let value: Value = serde_json::from_str(last_message)
        .map_err(|error| format!("protocol_invalid: 主管输出必须是单个动作 JSON：{error}"))?;
    let (schema_version, reason, expected_result) = {
        let object = value
            .as_object()
            .ok_or_else(|| "protocol_invalid: 主管输出必须是 JSON 对象。".to_string())?;
        let kind = object
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| "protocol_invalid: 缺少字符串字段 kind。".to_string())?;
        reject_unknown_fields(object.keys(), allowed_fields_for(kind))?;
        (
            required_string(object, "schema_version")?,
            required_string(object, "reason")?,
            required_string(object, "expected_result")?,
        )
    };
    let proposal = SupervisorActionProposalV1 {
        schema_version,
        action: serde_json::from_value(value)
            .map_err(|error| format!("protocol_invalid: 动作字段不符合 schema：{error}"))?,
        reason,
        expected_result,
    };
    if proposal.schema_version != SUPERVISOR_ACTION_PROPOSAL_SCHEMA_V1 {
        return Err(format!(
            "protocol_invalid: schema_version 必须是 {}",
            SUPERVISOR_ACTION_PROPOSAL_SCHEMA_V1
        ));
    }
    require_non_empty("reason", &proposal.reason)?;
    require_non_empty("expected_result", &proposal.expected_result)?;
    match &proposal.action {
        SupervisorActionKind::DispatchWorker { target } => {
            require_non_empty("target.node_id", &target.node_id)?;
            require_non_empty("target.work_item_id", &target.work_item_id)?;
        }
        SupervisorActionKind::InspectWorker { worker_id }
        | SupervisorActionKind::WaitWorker { worker_id } => {
            require_non_empty("worker_id", worker_id)?;
        }
        SupervisorActionKind::FollowUpWorker { worker_id, prompt } => {
            require_non_empty("worker_id", worker_id)?;
            require_non_empty("prompt", prompt)?;
        }
        SupervisorActionKind::Finalize { .. } => {}
        SupervisorActionKind::ReportUser { message } => require_non_empty("message", message)?,
    }
    Ok(proposal)
}

fn allowed_fields_for(kind: &str) -> &'static [&'static str] {
    match kind {
        "dispatch_worker" => &["schema_version", "kind", "target", "reason", "expected_result"],
        "inspect_worker" | "wait_worker" => {
            &["schema_version", "kind", "worker_id", "reason", "expected_result"]
        }
        "follow_up_worker" => &[
            "schema_version",
            "kind",
            "worker_id",
            "prompt",
            "reason",
            "expected_result",
        ],
        "finalize" => &["schema_version", "kind", "verdict", "reason", "expected_result"],
        "report_user" => &["schema_version", "kind", "message", "reason", "expected_result"],
        _ => &[],
    }
}

fn reject_unknown_fields<'a>(
    fields: impl Iterator<Item = &'a String>,
    allowed_fields: &[&str],
) -> Result<(), String> {
    let allowed = allowed_fields.iter().copied().collect::<BTreeSet<_>>();
    let unknown = fields
        .filter(|field| !allowed.contains(field.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "protocol_invalid: 动作包含不允许字段 {}",
            unknown.join("，")
        ))
    }
}

fn required_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("protocol_invalid: 缺少字符串字段 {field}。"))
}

fn require_non_empty(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("protocol_invalid: {label} 不能为空"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch() -> &'static str {
        r#"{
          "schema_version":"supervisor_action_proposal.v1",
          "kind":"dispatch_worker",
          "target":{"node_id":"node-1","work_item_id":"work-1"},
          "reason":"任务包已准备好",
          "expected_result":"获得受控 worker 回交"
        }"#
    }

    #[test]
    fn station3a_protocol_accepts_one_strict_dispatch_proposal() {
        let proposal = parse_supervisor_action_proposal(dispatch()).expect("valid proposal");
        assert_eq!(proposal.action.name(), "dispatch_worker");
    }

    #[test]
    fn station3a_protocol_accepts_each_frozen_action_shape() {
        for (kind, proposal) in [
            (
                "inspect_worker",
                r#"{"schema_version":"supervisor_action_proposal.v1","kind":"inspect_worker","worker_id":"worker-1","reason":"read","expected_result":"report"}"#,
            ),
            (
                "follow_up_worker",
                r#"{"schema_version":"supervisor_action_proposal.v1","kind":"follow_up_worker","worker_id":"worker-1","prompt":"show evidence","reason":"missing","expected_result":"evidence"}"#,
            ),
            (
                "wait_worker",
                r#"{"schema_version":"supervisor_action_proposal.v1","kind":"wait_worker","worker_id":"worker-1","reason":"running","expected_result":"state"}"#,
            ),
            (
                "finalize",
                r#"{"schema_version":"supervisor_action_proposal.v1","kind":"finalize","verdict":"blocked","reason":"risk","expected_result":"advisory"}"#,
            ),
            (
                "report_user",
                r#"{"schema_version":"supervisor_action_proposal.v1","kind":"report_user","message":"needs review","reason":"summary","expected_result":"visible report"}"#,
            ),
        ] {
            assert_eq!(
                parse_supervisor_action_proposal(proposal)
                    .expect("valid frozen action")
                    .action
                    .name(),
                kind
            );
        }
    }

    #[test]
    fn station3a_protocol_rejects_text_unknown_fields_and_permission_claims() {
        for invalid in [
            format!("主管说明：{}", dispatch()),
            format!("{}\n补充说明", dispatch()),
            dispatch().replace("\n        }", ",\"allowed_write\":[\"/tmp\"]\n        }"),
            dispatch().replace("\n        }", ",\"authorization_id\":\"self-approved\"\n        }"),
            dispatch().replace("\n        }", ",\"project_root\":\"/tmp\"\n        }"),
            dispatch().replace("\n        }", ",\"bypass\":true\n        }"),
            dispatch().replace(
                "\"work_item_id\":\"work-1\"",
                "\"work_item_id\":\"work-1\",\"allowed_write\":[\"/tmp\"]",
            ),
        ] {
            assert!(
                parse_supervisor_action_proposal(&invalid).is_err(),
                "must reject {invalid}"
            );
        }
    }

    #[test]
    fn station3a_protocol_rejects_missing_target_and_unknown_kind() {
        let missing_target = r#"{
          "schema_version":"supervisor_action_proposal.v1",
          "kind":"dispatch_worker",
          "reason":"x",
          "expected_result":"y"
        }"#;
        let unknown_kind = r#"{
          "schema_version":"supervisor_action_proposal.v1",
          "kind":"spawn_shell",
          "reason":"x",
          "expected_result":"y"
        }"#;
        assert!(parse_supervisor_action_proposal(missing_target).is_err());
        assert!(parse_supervisor_action_proposal(unknown_kind).is_err());
    }

    // P1-E 旧路退役：pilot 问人死胡同(RequestUserDecision)已退场——协议层不再认得这个 kind，
    // 应和其它未知动作一样被拒（P1-B resident 问答通道已替代它）。
    #[test]
    fn station3a_protocol_rejects_retired_request_user_decision_kind() {
        let retired = r#"{
          "schema_version":"supervisor_action_proposal.v1",
          "kind":"request_user_decision",
          "question":"continue?",
          "reason":"scope change",
          "expected_result":"decision"
        }"#;
        assert!(
            parse_supervisor_action_proposal(retired).is_err(),
            "已退役的 request_user_decision 应和未知动作一样被拒"
        );
    }

    #[test]
    fn station3a_v3_protocol_rejects_nested_target_worker_id_for_inspection() {
        let invalid = r#"{
          "schema_version":"supervisor_action_proposal.v1",
          "kind":"inspect_worker",
          "target":{"worker_id":"worker-1"},
          "reason":"读取回程",
          "expected_result":"获得证据"
        }"#;
        let error = parse_supervisor_action_proposal(invalid).expect_err("nested worker target must fail");
        assert!(error.contains("target"), "unexpected error: {error}");
    }
}
