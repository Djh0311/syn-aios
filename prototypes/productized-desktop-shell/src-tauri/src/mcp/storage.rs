// File-layer storage for canvas / run-state / audit / outbox.
// All director and subagent tools read and write through this module.
//
// Layout:
//   ~/Library/Application Support/CodexGovernanceWorkbench/canvas-v1/
//     canvas/<canvas_id>.json
//     runs/<run_id>/state.json
//     runs/<run_id>/audit.jsonl
//     runs/<run_id>/outbox/<node_id>.md

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_CANVAS: &str = "canvas-v1";
const SCHEMA_RUN: &str = "canvas-run-v1";
const SCHEMA_WORKFLOW_TEMPLATE: &str = "workflow-template-v1";

// ---------- types ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: String,
    pub role: String, // "director" | "subagent" (kept for back-compat / sealed run logic)
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    // Free-canvas authoring (plan A4): `kind` is an open node type and `data` is
    // a free payload (status/prompt/sandbox/custom fields). Opaque passthrough —
    // no interpretation, no execution. Optional + skip-if-none so pre-feature
    // canvases keep round-tripping unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    pub position: Position,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEdge {
    pub id: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasDefinition {
    pub schema_version: String,
    pub canvas_id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
    // Canvas surface scope (two-surfaces-one-engine plan P1/B): explicit
    // "experiment" | "project", persisted instead of derived from project_root,
    // so a "designed but not yet bound" project draft can exist. Opaque
    // passthrough — no interpretation. Optional + serde default = old canvases
    // round-trip unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

// ---------- workflow templates (plan B: 成熟模式保留) ----------
//
// Stores the workflow GRAPH itself (nodes/edges/node-data) + metadata, so a
// canvas that runs well can be saved as a reusable "mature pattern" and a new
// workflow can be instantiated from it. Deliberately separate from the memory
// `mature_pattern_store` (that holds memory patterns, not workflow graphs).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub schema_version: String, // "workflow-template-v1"
    pub template_id: String,
    pub title: String,
    #[serde(default)]
    pub scope: String, // "project" | "global"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_canvas_id: Option<String>,
    #[serde(default)]
    pub version: u32,
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateSummary {
    pub template_id: String,
    pub title: String,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasRunInbox {
    pub node_id: String,
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub dispatched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasRunOutboxPointer {
    pub node_id: String,
    pub outbox_path: String,
    pub summary: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasRunState {
    pub schema_version: String,
    pub run_id: String,
    pub canvas_id: String,
    pub goal: String,
    pub status: String, // "running" | "finished" | "aborted"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub busy_node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inbox: Option<CanvasRunInbox>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outbox: Option<CanvasRunOutboxPointer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abort_reason: Option<String>,
    pub started_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasAuditEvent {
    pub ts: String,
    pub actor: serde_json::Value,
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

// ---------- paths ----------

pub fn canvas_v1_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/yoyi".to_string());
    PathBuf::from(home).join("Library/Application Support/CodexGovernanceWorkbench/canvas-v1")
}

pub fn canvas_path(canvas_id: &str) -> PathBuf {
    canvas_v1_root()
        .join("canvas")
        .join(format!("{canvas_id}.json"))
}

pub fn run_dir(run_id: &str) -> PathBuf {
    canvas_v1_root().join("runs").join(run_id)
}

pub fn run_state_path(run_id: &str) -> PathBuf {
    run_dir(run_id).join("state.json")
}

pub fn run_audit_path(run_id: &str) -> PathBuf {
    run_dir(run_id).join("audit.jsonl")
}

pub fn run_outbox_path(run_id: &str, node_id: &str) -> PathBuf {
    run_dir(run_id).join("outbox").join(format!("{node_id}.md"))
}

// ---------- canvas ----------

pub fn load_canvas(canvas_id: &str) -> Result<CanvasDefinition, String> {
    let p = canvas_path(canvas_id);
    let text = fs::read_to_string(&p).map_err(|e| format!("读画布失败 {}：{e}", p.display()))?;
    let canvas: CanvasDefinition = serde_json::from_str(&text)
        .map_err(|e| format!("画布 JSON 解析失败 {}：{e}", p.display()))?;
    if canvas.schema_version != SCHEMA_CANVAS {
        return Err(format!(
            "画布 schema_version={} 期望 {}",
            canvas.schema_version, SCHEMA_CANVAS
        ));
    }
    Ok(canvas)
}

pub fn save_canvas(canvas: &CanvasDefinition) -> Result<(), String> {
    let p = canvas_path(&canvas.canvas_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("建目录失败 {}：{e}", parent.display()))?;
    }
    write_atomic(&p, &serde_json::to_string_pretty(canvas).unwrap())
}

// ---------- workflow templates ----------

pub fn workflow_template_dir() -> PathBuf {
    canvas_v1_root().join("workflow-templates")
}

pub fn workflow_template_path(template_id: &str) -> PathBuf {
    workflow_template_dir().join(format!("{template_id}.json"))
}

pub fn save_workflow_template(template: &WorkflowTemplate) -> Result<(), String> {
    if template.template_id.trim().is_empty() {
        return Err("workflow template 缺 template_id".to_string());
    }
    let p = workflow_template_path(&template.template_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("建目录失败 {}：{e}", parent.display()))?;
    }
    write_atomic(&p, &serde_json::to_string_pretty(template).unwrap())
}

pub fn load_workflow_template(template_id: &str) -> Result<WorkflowTemplate, String> {
    let p = workflow_template_path(template_id);
    let text = fs::read_to_string(&p).map_err(|e| format!("读模板失败 {}：{e}", p.display()))?;
    let template: WorkflowTemplate = serde_json::from_str(&text)
        .map_err(|e| format!("模板 JSON 解析失败 {}：{e}", p.display()))?;
    if template.schema_version != SCHEMA_WORKFLOW_TEMPLATE {
        return Err(format!(
            "模板 schema_version={} 期望 {}",
            template.schema_version, SCHEMA_WORKFLOW_TEMPLATE
        ));
    }
    Ok(template)
}

pub fn list_workflow_templates() -> Result<Vec<WorkflowTemplateSummary>, String> {
    let dir = workflow_template_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(&dir).map_err(|e| format!("读模板目录失败 {}：{e}", dir.display()))?;
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(template) = serde_json::from_str::<WorkflowTemplate>(&text) else {
            continue;
        };
        out.push(WorkflowTemplateSummary {
            template_id: template.template_id,
            title: template.title,
            scope: template.scope,
            project_root: template.project_root,
            node_count: template.nodes.len(),
            edge_count: template.edges.len(),
            created_at: template.created_at,
            updated_at: template.updated_at,
        });
    }
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.title.cmp(&b.title)));
    Ok(out)
}

pub fn delete_workflow_template(template_id: &str) -> Result<(), String> {
    let p = workflow_template_path(template_id);
    if !p.exists() {
        return Ok(());
    }
    fs::remove_file(&p).map_err(|e| format!("删模板失败 {}：{e}", p.display()))
}

// ---------- run state ----------

pub fn load_run_state(run_id: &str) -> Result<CanvasRunState, String> {
    let p = run_state_path(run_id);
    let text =
        fs::read_to_string(&p).map_err(|e| format!("读 run state 失败 {}：{e}", p.display()))?;
    let st: CanvasRunState = serde_json::from_str(&text)
        .map_err(|e| format!("run state JSON 解析失败 {}：{e}", p.display()))?;
    if st.schema_version != SCHEMA_RUN {
        return Err(format!(
            "run state schema_version={} 期望 {}",
            st.schema_version, SCHEMA_RUN
        ));
    }
    Ok(st)
}

pub fn save_run_state(state: &CanvasRunState) -> Result<(), String> {
    let p = run_state_path(&state.run_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("建目录失败 {}：{e}", parent.display()))?;
    }
    write_atomic(&p, &serde_json::to_string_pretty(state).unwrap())
}

pub fn create_run(run_id: &str, canvas_id: &str, goal: &str) -> Result<CanvasRunState, String> {
    let now = iso_now();
    let st = CanvasRunState {
        schema_version: SCHEMA_RUN.to_string(),
        run_id: run_id.to_string(),
        canvas_id: canvas_id.to_string(),
        goal: goal.to_string(),
        status: "running".to_string(),
        busy_node_id: None,
        inbox: None,
        outbox: None,
        finish_summary: None,
        abort_reason: None,
        started_at: now.clone(),
        updated_at: now,
    };
    save_run_state(&st)?;
    fs::create_dir_all(run_dir(run_id).join("outbox"))
        .map_err(|e| format!("建 outbox 目录失败：{e}"))?;
    Ok(st)
}

// ---------- audit ----------

pub fn append_audit(run_id: &str, event: &CanvasAuditEvent) -> Result<(), String> {
    let p = run_audit_path(run_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("建目录失败 {}：{e}", parent.display()))?;
    }
    let line = serde_json::to_string(event).map_err(|e| format!("audit 序列化失败：{e}"))?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|e| format!("打开 audit 失败 {}：{e}", p.display()))?;
    writeln!(f, "{line}").map_err(|e| format!("写 audit 失败：{e}"))?;
    Ok(())
}

pub fn read_recent_audit(run_id: &str, n: usize) -> Result<Vec<CanvasAuditEvent>, String> {
    let p = run_audit_path(run_id);
    if !p.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&p).map_err(|e| format!("读 audit 失败 {}：{e}", p.display()))?;
    let mut events = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<CanvasAuditEvent>(trimmed) {
            Ok(ev) => events.push(ev),
            Err(_) => continue, // skip malformed lines
        }
    }
    let len = events.len();
    if len > n {
        Ok(events.split_off(len - n))
    } else {
        Ok(events)
    }
}

// ---------- outbox ----------

pub fn write_outbox(run_id: &str, node_id: &str, content: &str) -> Result<PathBuf, String> {
    let p = run_outbox_path(run_id, node_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("建目录失败 {}：{e}", parent.display()))?;
    }
    write_atomic(&p, content)?;
    Ok(p)
}

pub fn read_outbox_file(run_id: &str, node_id: &str) -> Result<String, String> {
    let p = run_outbox_path(run_id, node_id);
    fs::read_to_string(&p).map_err(|e| format!("读 outbox 失败 {}：{e}", p.display()))
}

// ---------- helpers ----------

fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let tmp = path.with_extension(format!(
        "tmp-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    {
        let mut f =
            fs::File::create(&tmp).map_err(|e| format!("建临时文件失败 {}：{e}", tmp.display()))?;
        f.write_all(content.as_bytes())
            .map_err(|e| format!("写临时文件失败：{e}"))?;
        f.sync_all().map_err(|e| format!("同步失败：{e}"))?;
    }
    fs::rename(&tmp, path).map_err(|e| format!("替换文件失败 {}：{e}", path.display()))
}

pub fn iso_now() -> String {
    // RFC 3339 / ISO 8601 in UTC, second precision. Manual formatter so we
    // don't pull chrono.
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (year, month, day, hour, minute, second) = unix_to_civil(secs as i64);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

fn unix_to_civil(secs: i64) -> (i32, u32, u32, u32, u32, u32) {
    // Howard Hinnant's days_from_civil inverse — handles year 1970+.
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400) as u32;
    let hour = rem / 3600;
    let minute = (rem % 3600) / 60;
    let second = rem % 60;

    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = (y + if m <= 2 { 1 } else { 0 }) as i32;
    (year, m, d, hour, minute, second)
}
