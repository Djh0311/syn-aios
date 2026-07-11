// MCP tool implementations.
// Director-side 6 tools (step 3) + subagent-side 2 tools (step 4) all live here.
// All state changes go through the storage module; no in-memory state on the
// server side.

use serde_json::{json, Value};

use super::storage::{
    self, CanvasAuditEvent, CanvasRunInbox, CanvasRunOutboxPointer, CanvasRunState,
};
use super::{McpRole, McpServerConfig};

pub fn list_tools(role: McpRole) -> Value {
    let tools: Vec<Value> = match role {
        McpRole::Director => vec![
            tool_def(
                "list_team",
                "看车间状态：canvas + 当前 run + 最近审计",
                json!({ "type": "object", "properties": {}, "additionalProperties": false }),
            ),
            tool_def(
                "dispatch",
                "派活给某个子 agent 节点",
                json!({
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" },
                        "task": { "type": "string" },
                        "scope": { "type": "string" }
                    },
                    "required": ["node_id", "task"],
                    "additionalProperties": false
                }),
            ),
            tool_def(
                "read_outbox",
                "读某个子 agent 上次交回的结果（先 summary，按需展开 content）",
                json!({
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" },
                        "expand_content": { "type": "boolean", "default": false }
                    },
                    "required": ["node_id"],
                    "additionalProperties": false
                }),
            ),
            tool_def(
                "recycle",
                "对某个子 agent 上一次派活做出判断：pass / changes / reject",
                json!({
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string" },
                        "verdict": { "type": "string", "enum": ["pass", "changes", "reject"] },
                        "notes": { "type": "string" }
                    },
                    "required": ["node_id", "verdict", "notes"],
                    "additionalProperties": false
                }),
            ),
            tool_def(
                "stop",
                "拍停某个子 agent 当前任务",
                json!({
                    "type": "object",
                    "properties": { "node_id": { "type": "string" } },
                    "required": ["node_id"],
                    "additionalProperties": false
                }),
            ),
            tool_def(
                "finish",
                "宣布目标完成，整个车间收工",
                json!({
                    "type": "object",
                    "properties": { "summary": { "type": "string" } },
                    "required": ["summary"],
                    "additionalProperties": false
                }),
            ),
        ],
        McpRole::Subagent => vec![
            tool_def(
                "submit_outbox",
                "交活：content 落到 outbox 文件，summary 给主管看",
                json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string" },
                        "summary": { "type": "string" }
                    },
                    "required": ["content", "summary"],
                    "additionalProperties": false
                }),
            ),
            tool_def(
                "report_blocked",
                "卡住，请主管定夺",
                json!({
                    "type": "object",
                    "properties": { "reason": { "type": "string" } },
                    "required": ["reason"],
                    "additionalProperties": false
                }),
            ),
        ],
        // `supervisor_orchestrator` has a deliberately separate module and sidecar.
        // This empty arm only keeps the legacy canvas role matcher exhaustive.
        McpRole::SupervisorOrchestrator => vec![],
    };
    json!({ "tools": tools })
}

fn tool_def(name: &str, description: &str, schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": schema
    })
}

pub fn call_tool(config: &McpServerConfig, params: Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "tools/call 缺少 name".to_string())?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    let allowed = match config.role {
        McpRole::Director => matches!(
            name,
            "list_team" | "dispatch" | "read_outbox" | "recycle" | "stop" | "finish"
        ),
        McpRole::Subagent => matches!(name, "submit_outbox" | "report_blocked"),
        McpRole::SupervisorOrchestrator => false,
    };
    if !allowed {
        return Err(format!("工具 {name} 不在 {:?} 的可用集合里", config.role));
    }

    let result_text = match (config.role, name) {
        (McpRole::Director, "list_team") => director_list_team(config),
        (McpRole::Director, "dispatch") => director_dispatch(config, &arguments),
        (McpRole::Director, "read_outbox") => director_read_outbox(config, &arguments),
        (McpRole::Director, "recycle") => director_recycle(config, &arguments),
        (McpRole::Director, "stop") => director_stop(config, &arguments),
        (McpRole::Director, "finish") => director_finish(config, &arguments),
        (McpRole::Subagent, "submit_outbox") => subagent_submit_outbox(config, &arguments),
        (McpRole::Subagent, "report_blocked") => subagent_report_blocked(config, &arguments),
        _ => Err(format!("未实现：{name}")),
    }?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": result_text
        }]
    }))
}

// =============================================================
// Director tools
// =============================================================

fn director_list_team(config: &McpServerConfig) -> Result<String, String> {
    let run = storage::load_run_state(&config.run_id)?;
    let canvas = storage::load_canvas(&run.canvas_id)?;
    let recent_audit = storage::read_recent_audit(&config.run_id, 20)?;
    let view = json!({
        "canvas": canvas,
        "run": run,
        "recent_audit": recent_audit,
    });
    Ok(serde_json::to_string_pretty(&view).unwrap())
}

fn director_dispatch(config: &McpServerConfig, args: &Value) -> Result<String, String> {
    let node_id = require_str(args, "node_id")?;
    let task = require_str(args, "task")?;
    let scope = optional_str(args, "scope");

    let mut run = storage::load_run_state(&config.run_id)?;
    if run.status != "running" {
        return Err(format!("run 当前状态 {} 不允许派活", run.status));
    }
    if let Some(busy) = run.busy_node_id.as_ref() {
        return Err(format!(
            "v1 单线：{} 还在跑，先 recycle 或 stop 再派下一个",
            busy
        ));
    }
    let canvas = storage::load_canvas(&run.canvas_id)?;
    let target = canvas
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| format!("画布里没有节点 {node_id}"))?;
    if target.role != "subagent" {
        return Err(format!(
            "只能派给子 agent，节点 {} 角色是 {}",
            node_id, target.role
        ));
    }
    if target.session_id.is_none() {
        return Err(format!("节点 {} 还没挂 codex 会话", node_id));
    }

    let now = storage::iso_now();
    run.busy_node_id = Some(node_id.to_string());
    run.inbox = Some(CanvasRunInbox {
        node_id: node_id.to_string(),
        task: task.to_string(),
        scope: scope.clone(),
        dispatched_at: now.clone(),
    });
    run.outbox = None; // any prior outbox is now stale
    run.updated_at = now.clone();
    storage::save_run_state(&run)?;

    storage::append_audit(
        &config.run_id,
        &CanvasAuditEvent {
            ts: now,
            actor: json!({ "kind": "director", "node_id": director_node_id(&canvas) }),
            action: "dispatch".to_string(),
            target_node_id: Some(node_id.to_string()),
            payload: Some(json!({ "task": task, "scope": scope })),
        },
    )?;

    Ok(format!("已派活给 {node_id}，等子 agent 交回。"))
}

fn director_read_outbox(config: &McpServerConfig, args: &Value) -> Result<String, String> {
    let node_id = require_str(args, "node_id")?;
    let expand = args
        .get("expand_content")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let run = storage::load_run_state(&config.run_id)?;
    let outbox = run
        .outbox
        .as_ref()
        .filter(|o| o.node_id == node_id)
        .ok_or_else(|| format!("{node_id} 还没交活，没东西读"))?;

    let mut payload = json!({
        "node_id": outbox.node_id,
        "summary": outbox.summary,
        "submitted_at": outbox.submitted_at,
        "outbox_path": outbox.outbox_path,
    });
    if expand {
        let content = storage::read_outbox_file(&config.run_id, &node_id)?;
        payload["content"] = json!(content);
    }
    Ok(serde_json::to_string_pretty(&payload).unwrap())
}

fn director_recycle(config: &McpServerConfig, args: &Value) -> Result<String, String> {
    let node_id = require_str(args, "node_id")?;
    let verdict = require_str(args, "verdict")?;
    let notes = require_str(args, "notes")?;
    if !matches!(verdict.as_str(), "pass" | "changes" | "reject") {
        return Err(format!("verdict 不认识: {verdict}"));
    }

    let mut run = storage::load_run_state(&config.run_id)?;
    if run.busy_node_id.as_deref() != Some(node_id.as_str()) {
        // tolerate idempotent recycle when outbox is from this node
        let allow_via_outbox = run
            .outbox
            .as_ref()
            .map(|o| o.node_id == node_id)
            .unwrap_or(false);
        if !allow_via_outbox {
            return Err(format!("{node_id} 当前不是在跑的节点，无法 recycle"));
        }
    }

    let canvas = storage::load_canvas(&run.canvas_id)?;
    let now = storage::iso_now();
    run.busy_node_id = None;
    run.inbox = None;
    run.outbox = None; // outbox consumed by recycle decision
    run.updated_at = now.clone();
    storage::save_run_state(&run)?;

    storage::append_audit(
        &config.run_id,
        &CanvasAuditEvent {
            ts: now,
            actor: json!({ "kind": "director", "node_id": director_node_id(&canvas) }),
            action: "recycle".to_string(),
            target_node_id: Some(node_id.to_string()),
            payload: Some(json!({ "verdict": verdict, "notes": notes })),
        },
    )?;

    Ok(format!("已 recycle {node_id}: verdict={verdict}"))
}

fn director_stop(config: &McpServerConfig, args: &Value) -> Result<String, String> {
    let node_id = require_str(args, "node_id")?;

    let mut run = storage::load_run_state(&config.run_id)?;
    if run.busy_node_id.as_deref() != Some(node_id.as_str()) {
        return Err(format!("{node_id} 当前没在跑，无需 stop"));
    }
    let canvas = storage::load_canvas(&run.canvas_id)?;
    let now = storage::iso_now();
    run.busy_node_id = None;
    run.inbox = None;
    run.updated_at = now.clone();
    storage::save_run_state(&run)?;

    storage::append_audit(
        &config.run_id,
        &CanvasAuditEvent {
            ts: now,
            actor: json!({ "kind": "director", "node_id": director_node_id(&canvas) }),
            action: "stop".to_string(),
            target_node_id: Some(node_id.to_string()),
            payload: None,
        },
    )?;

    Ok(format!("已拍停 {node_id}"))
}

fn director_finish(config: &McpServerConfig, args: &Value) -> Result<String, String> {
    let summary = require_str(args, "summary")?;
    let mut run = storage::load_run_state(&config.run_id)?;
    if run.status == "finished" {
        return Err("run 已经 finished".to_string());
    }
    let canvas = storage::load_canvas(&run.canvas_id)?;
    let now = storage::iso_now();
    run.status = "finished".to_string();
    run.finish_summary = Some(summary.clone());
    run.busy_node_id = None;
    run.inbox = None;
    run.updated_at = now.clone();
    storage::save_run_state(&run)?;

    storage::append_audit(
        &config.run_id,
        &CanvasAuditEvent {
            ts: now,
            actor: json!({ "kind": "director", "node_id": director_node_id(&canvas) }),
            action: "finish".to_string(),
            target_node_id: None,
            payload: Some(json!({ "summary": summary })),
        },
    )?;

    Ok("已宣布完成。".to_string())
}

// =============================================================
// Subagent tools
// =============================================================

fn subagent_submit_outbox(config: &McpServerConfig, args: &Value) -> Result<String, String> {
    let node_id = config
        .node_id
        .clone()
        .ok_or_else(|| "子 agent 模式必须有 node_id".to_string())?;
    let content = require_str(args, "content")?;
    let summary = require_str(args, "summary")?;

    let mut run = storage::load_run_state(&config.run_id)?;
    if run.status != "running" {
        return Err(format!("run 当前状态 {} 不允许 submit_outbox", run.status));
    }
    if run.busy_node_id.as_deref() != Some(node_id.as_str()) {
        return Err(format!(
            "{} 当前不是在跑的节点（busy={:?}），可能已被 stop 或 recycle",
            node_id, run.busy_node_id
        ));
    }

    let path = storage::write_outbox(&config.run_id, &node_id, &content)?;
    let now = storage::iso_now();
    run.outbox = Some(CanvasRunOutboxPointer {
        node_id: node_id.clone(),
        outbox_path: path.display().to_string(),
        summary: summary.clone(),
        submitted_at: now.clone(),
    });
    // busy_node_id 不在此清空——主管 recycle 才视为本轮真正闭环。
    // 这样万一主管想再追问 / 让子继续干，状态仍是 busy=node_id。
    run.updated_at = now.clone();
    storage::save_run_state(&run)?;

    storage::append_audit(
        &config.run_id,
        &CanvasAuditEvent {
            ts: now,
            actor: json!({ "kind": "subagent", "node_id": node_id }),
            action: "submit_outbox".to_string(),
            target_node_id: Some(node_id.clone()),
            payload: Some(json!({
                "summary": summary,
                "outbox_path": path.display().to_string(),
                "content_bytes": content.len(),
            })),
        },
    )?;

    Ok(format!(
        "已交活：summary 长度 {}，content 长度 {}，落到 {}",
        summary.len(),
        content.len(),
        path.display()
    ))
}

fn subagent_report_blocked(config: &McpServerConfig, args: &Value) -> Result<String, String> {
    let node_id = config
        .node_id
        .clone()
        .ok_or_else(|| "子 agent 模式必须有 node_id".to_string())?;
    let reason = require_str(args, "reason")?;

    let mut run = storage::load_run_state(&config.run_id)?;
    if run.status != "running" {
        return Err(format!("run 当前状态 {} 不允许 report_blocked", run.status));
    }
    if run.busy_node_id.as_deref() != Some(node_id.as_str()) {
        return Err(format!(
            "{} 当前不是在跑的节点（busy={:?}）",
            node_id, run.busy_node_id
        ));
    }

    let now = storage::iso_now();
    // 与 submit_outbox 不同：blocked 视为本轮中止，清空 busy 让主管来定夺。
    run.busy_node_id = None;
    run.inbox = None;
    run.updated_at = now.clone();
    storage::save_run_state(&run)?;

    storage::append_audit(
        &config.run_id,
        &CanvasAuditEvent {
            ts: now,
            actor: json!({ "kind": "subagent", "node_id": node_id }),
            action: "report_blocked".to_string(),
            target_node_id: Some(node_id.clone()),
            payload: Some(json!({ "reason": reason })),
        },
    )?;

    Ok(format!("已上报阻塞：{reason}"))
}

// =============================================================
// helpers
// =============================================================

fn require_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("缺少必填字段 {key}"))
}

fn optional_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

fn director_node_id(canvas: &storage::CanvasDefinition) -> String {
    canvas
        .nodes
        .iter()
        .find(|n| n.role == "director")
        .map(|n| n.id.clone())
        .unwrap_or_else(|| "director".to_string())
}

// `CanvasRunState` is currently only built and read via storage; suppress unused-import lint
// when sub-modules don't import it directly.
#[allow(dead_code)]
fn _silence_unused(_: &CanvasRunState, _: &CanvasRunOutboxPointer) {}
