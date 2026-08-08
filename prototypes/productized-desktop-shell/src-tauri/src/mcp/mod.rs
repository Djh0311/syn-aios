// Editable Canvas v1 — MCP server entry.
// 决策：decisions/2026-05-31-editable-canvas-codex-as-director-v1.md
//
// 形状：
//   每个 codex 会话 spawn 一个本 binary 子进程，通过 stdio 跑 JSON-RPC 2.0。
//   身份在启动参数里 baked in：--role director|subagent --run-id <id> [--node-id <id>]
//   server 不维护内存状态；canvas/state/audit 一律读写文件层。
//
// 当前阶段：skeleton + identity routing。所有工具调用先返回占位错误，第 3/4 步逐个填实。

pub(crate) mod capability_registry;
pub mod commands;
pub(crate) mod event_audit_boundary;
pub(crate) mod execution_grant;
pub(crate) mod identity_kernel;
pub mod orchestrator;
pub(crate) mod path_guard;
mod protocol;
pub mod storage;
pub(crate) mod supervisor_conversation_binding;
pub(crate) mod supervisor_orchestrator;
mod tools;

use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpRole {
    Director,
    Subagent,
    SupervisorOrchestrator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupervisorQuotaLimits {
    pub max_active_workers: usize,
    pub max_follow_ups_per_worker: usize,
    pub max_runtime_minutes: i64,
}

#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub role: McpRole,
    pub run_id: String,
    pub node_id: Option<String>,
    pub supervisor_workflow_state_path: Option<PathBuf>,
    pub supervisor_quota_limits: Option<SupervisorQuotaLimits>,
    /// Host-issued, child-only relay details for one `knowledge_open` turn.
    /// This is never sourced from a tool argument or frontend request.
    pub(crate) knowledge_open_relay:
        Option<crate::knowledge_open_relay::KnowledgeOpenRelayMcpConfig>,
}

pub fn run_mcp_server_cli(args: Vec<String>) -> Result<(), String> {
    let config = parse_args(&args)?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("stdin read failed: {e}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(req) => handle_request(&config, req),
            Err(parse_err) => Some(JsonRpcResponse::error(
                serde_json::Value::Null,
                JsonRpcError::parse_error(&parse_err.to_string()),
            )),
        };
        if let Some(resp) = response {
            let payload = serde_json::to_string(&resp)
                .map_err(|e| format!("serialize response failed: {e}"))?;
            writeln!(writer, "{payload}").map_err(|e| format!("stdout write failed: {e}"))?;
            writer
                .flush()
                .map_err(|e| format!("stdout flush failed: {e}"))?;
        }
    }
    Ok(())
}

fn parse_args(args: &[String]) -> Result<McpServerConfig, String> {
    let mut role: Option<McpRole> = None;
    let mut run_id: Option<String> = None;
    let mut node_id: Option<String> = None;
    let mut supervisor_workflow_state_path: Option<PathBuf> = None;
    let mut max_active_workers: Option<usize> = None;
    let mut max_follow_ups_per_worker: Option<usize> = None;
    let mut max_runtime_minutes: Option<i64> = None;
    let mut knowledge_open_relay_endpoint: Option<String> = None;
    let mut knowledge_open_relay_grant: Option<String> = None;
    let mut knowledge_open_relay_turn_id: Option<String> = None;
    let mut knowledge_open_relay_project_id: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let key = args[i].as_str();
        let val = args.get(i + 1).cloned();
        match key {
            "--role" => {
                let raw = val.ok_or_else(|| "--role 缺少值".to_string())?;
                role = Some(match raw.as_str() {
                    "director" => McpRole::Director,
                    "subagent" => McpRole::Subagent,
                    "supervisor_orchestrator" => McpRole::SupervisorOrchestrator,
                    other => return Err(format!("--role 不认识: {other}")),
                });
                i += 2;
            }
            "--run-id" => {
                run_id = Some(val.ok_or_else(|| "--run-id 缺少值".to_string())?);
                i += 2;
            }
            "--node-id" => {
                node_id = Some(val.ok_or_else(|| "--node-id 缺少值".to_string())?);
                i += 2;
            }
            "--workflow-state-path" => {
                supervisor_workflow_state_path = Some(PathBuf::from(
                    val.ok_or_else(|| "--workflow-state-path 缺少值".to_string())?,
                ));
                i += 2;
            }
            "--max-active-workers" => {
                let raw = val.ok_or_else(|| "--max-active-workers 缺少值".to_string())?;
                max_active_workers = Some(
                    raw.parse()
                        .map_err(|_| "--max-active-workers 必须是非负整数".to_string())?,
                );
                i += 2;
            }
            "--max-follow-ups-per-worker" => {
                let raw = val.ok_or_else(|| "--max-follow-ups-per-worker 缺少值".to_string())?;
                max_follow_ups_per_worker = Some(
                    raw.parse()
                        .map_err(|_| "--max-follow-ups-per-worker 必须是非负整数".to_string())?,
                );
                i += 2;
            }
            "--max-runtime-minutes" => {
                let raw = val.ok_or_else(|| "--max-runtime-minutes 缺少值".to_string())?;
                max_runtime_minutes = Some(
                    raw.parse()
                        .map_err(|_| "--max-runtime-minutes 必须是整数".to_string())?,
                );
                i += 2;
            }
            "--knowledge-open-relay-endpoint" => {
                if knowledge_open_relay_endpoint.is_some() {
                    return Err("knowledge_open relay 参数不可重复".to_string());
                }
                knowledge_open_relay_endpoint =
                    Some(val.ok_or_else(|| "--knowledge-open-relay-endpoint 缺少值".to_string())?);
                i += 2;
            }
            "--knowledge-open-relay-grant" => {
                if knowledge_open_relay_grant.is_some() {
                    return Err("knowledge_open relay 参数不可重复".to_string());
                }
                knowledge_open_relay_grant =
                    Some(val.ok_or_else(|| "--knowledge-open-relay-grant 缺少值".to_string())?);
                i += 2;
            }
            "--knowledge-open-relay-turn-id" => {
                if knowledge_open_relay_turn_id.is_some() {
                    return Err("knowledge_open relay 参数不可重复".to_string());
                }
                knowledge_open_relay_turn_id =
                    Some(val.ok_or_else(|| "--knowledge-open-relay-turn-id 缺少值".to_string())?);
                i += 2;
            }
            "--knowledge-open-relay-project-id" => {
                if knowledge_open_relay_project_id.is_some() {
                    return Err("knowledge_open relay 参数不可重复".to_string());
                }
                knowledge_open_relay_project_id = Some(
                    val.ok_or_else(|| "--knowledge-open-relay-project-id 缺少值".to_string())?,
                );
                i += 2;
            }
            other => return Err(format!("不认识的参数: {other}")),
        }
    }
    let role = role.ok_or_else(|| "缺少 --role".to_string())?;
    let run_id = run_id.ok_or_else(|| "缺少 --run-id".to_string())?;
    if role == McpRole::Subagent && node_id.is_none() {
        return Err("子 agent 模式必须提供 --node-id".to_string());
    }
    let relay_argument_count = [
        knowledge_open_relay_endpoint.is_some(),
        knowledge_open_relay_grant.is_some(),
        knowledge_open_relay_turn_id.is_some(),
        knowledge_open_relay_project_id.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    let supervisor_quota_limits = if role == McpRole::SupervisorOrchestrator {
        let limits = SupervisorQuotaLimits {
            max_active_workers: max_active_workers
                .ok_or_else(|| "主管编排模式必须提供 --max-active-workers".to_string())?,
            max_follow_ups_per_worker: max_follow_ups_per_worker
                .ok_or_else(|| "主管编排模式必须提供 --max-follow-ups-per-worker".to_string())?,
            max_runtime_minutes: max_runtime_minutes
                .ok_or_else(|| "主管编排模式必须提供 --max-runtime-minutes".to_string())?,
        };
        if limits.max_active_workers == 0 || limits.max_runtime_minutes <= 0 {
            return Err("主管编排配额必须为正数".to_string());
        }
        if supervisor_workflow_state_path.is_none() {
            return Err("主管编排模式必须提供 --workflow-state-path".to_string());
        }
        Some(limits)
    } else {
        if supervisor_workflow_state_path.is_some()
            || max_active_workers.is_some()
            || max_follow_ups_per_worker.is_some()
            || max_runtime_minutes.is_some()
            || relay_argument_count > 0
        {
            return Err("主管编排参数只允许 --role supervisor_orchestrator 使用".to_string());
        }
        None
    };
    let knowledge_open_relay = match relay_argument_count {
        0 => None,
        4 if role == McpRole::SupervisorOrchestrator => Some(
            crate::knowledge_open_relay::KnowledgeOpenRelayMcpConfig::from_mcp_arguments(
                knowledge_open_relay_endpoint.expect("count checked"),
                knowledge_open_relay_grant.expect("count checked"),
                knowledge_open_relay_turn_id.expect("count checked"),
                knowledge_open_relay_project_id.expect("count checked"),
            )?,
        ),
        _ => return Err("knowledge_open relay 参数必须由宿主完整成组提供".to_string()),
    };
    Ok(McpServerConfig {
        role,
        run_id,
        node_id,
        supervisor_workflow_state_path,
        supervisor_quota_limits,
        knowledge_open_relay,
    })
}

fn handle_request(config: &McpServerConfig, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = req.id.clone();
    let is_notification = id.is_null();
    let result = match req.method.as_str() {
        "initialize" => Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": match config.role {
                    McpRole::SupervisorOrchestrator => "codex-governance-supervisor-orchestrator-mcp",
                    _ => "codex-governance-canvas-mcp"
                },
                "version": "0.1.0"
            }
        })),
        "notifications/initialized" => {
            return None;
        }
        "tools/list" => match config.role {
            McpRole::SupervisorOrchestrator => Ok(supervisor_orchestrator::list_tools(config)),
            _ => Ok(tools::list_tools(config.role)),
        },
        "tools/call" => match config.role {
            McpRole::SupervisorOrchestrator => supervisor_orchestrator::call_tool(
                config,
                req.params.unwrap_or(serde_json::Value::Null),
            ),
            _ => tools::call_tool(config, req.params.unwrap_or(serde_json::Value::Null)),
        }
        .map_err(JsonRpcError::tool_error),
        "ping" => Ok(serde_json::json!({})),
        other => Err(JsonRpcError::method_not_found(other)),
    };
    if is_notification {
        return None;
    }
    Some(match result {
        Ok(value) => JsonRpcResponse::ok(id, value),
        Err(err) => JsonRpcResponse::error(id, err),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supervisor_relay_args() -> Vec<String> {
        [
            "--role",
            "supervisor_orchestrator",
            "--run-id",
            "supervisor-conversation:relay-test",
            "--workflow-state-path",
            "/tmp/syn-relay-workflow-state.json",
            "--max-active-workers",
            "1",
            "--max-follow-ups-per-worker",
            "0",
            "--max-runtime-minutes",
            "1",
            "--knowledge-open-relay-endpoint",
            "/tmp/syn-knowledge-open-relay-test.sock",
            "--knowledge-open-relay-grant",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "--knowledge-open-relay-turn-id",
            "turn:relay-test",
            "--knowledge-open-relay-project-id",
            "project:relay-test",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    #[test]
    fn supervisor_relay_arguments_must_be_complete_and_nonduplicated() {
        assert!(parse_args(&supervisor_relay_args()).is_ok());

        for (flag, value) in [
            (
                "--knowledge-open-relay-endpoint",
                "/tmp/syn-knowledge-open-relay-overwrite.sock",
            ),
            (
                "--knowledge-open-relay-grant",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            ("--knowledge-open-relay-turn-id", "turn:relay-overwrite"),
            (
                "--knowledge-open-relay-project-id",
                "project:relay-overwrite",
            ),
        ] {
            let mut arguments = supervisor_relay_args();
            arguments.extend([flag.to_string(), value.to_string()]);
            assert!(
                parse_args(&arguments).is_err(),
                "duplicate {flag} must fail closed instead of replacing host-issued relay input"
            );
        }
    }
}
