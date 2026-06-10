// Editable Canvas v1 — MCP server entry.
// 决策：decisions/2026-05-31-editable-canvas-codex-as-director-v1.md
//
// 形状：
//   每个 codex 会话 spawn 一个本 binary 子进程，通过 stdio 跑 JSON-RPC 2.0。
//   身份在启动参数里 baked in：--role director|subagent --run-id <id> [--node-id <id>]
//   server 不维护内存状态；canvas/state/audit 一律读写文件层。
//
// 当前阶段：skeleton + identity routing。所有工具调用先返回占位错误，第 3/4 步逐个填实。

pub mod commands;
pub mod orchestrator;
mod protocol;
pub mod storage;
mod tools;

use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use std::io::{self, BufRead, BufWriter, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpRole {
    Director,
    Subagent,
}

#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub role: McpRole,
    pub run_id: String,
    pub node_id: Option<String>,
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
            other => return Err(format!("不认识的参数: {other}")),
        }
    }
    let role = role.ok_or_else(|| "缺少 --role".to_string())?;
    let run_id = run_id.ok_or_else(|| "缺少 --run-id".to_string())?;
    if role == McpRole::Subagent && node_id.is_none() {
        return Err("子 agent 模式必须提供 --node-id".to_string());
    }
    Ok(McpServerConfig {
        role,
        run_id,
        node_id,
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
                "name": "codex-governance-canvas-mcp",
                "version": "0.1.0"
            }
        })),
        "notifications/initialized" => {
            return None;
        }
        "tools/list" => Ok(tools::list_tools(config.role)),
        "tools/call" => tools::call_tool(config, req.params.unwrap_or(serde_json::Value::Null))
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
