// B1·全局主管 agent：读本轮口供+证据+所批方案 → 出复核意见（advisory）。
//
// 任务包：tasks/2026-07-07-phase-b1-global-supervisor-review-on-reports-v1.md
// 决策正本：decisions/2026-07-07-phase-b-advisory-supervisor-and-secretary-v1.md
//
// 安全属性（定稿四条·安全死线）：
// - **意见不是闸**：本模块不因 verdict 改任何工作流/链/工作项状态；建议动作只是字段，按钮由用户点；
// - **结构性只读**：LM 通道 = 现成 `readonly_codex_consult`（read-only 沙箱·写盘根空·06-25 豁免先例，只调不改）；
// - **输入全从盘读**（principles §4 事实核心落盘）：方案(proposal store)/链轮(workflow_chain_runs)/
//   口供(audit_events·worker_structured_report_recorded)/任务节点(顶层 nodes `:node:task:` 前缀)——
//   全走现成 `read_workflow_state_value` + 既有数组投影（读法与 c4_c6 自家 has_worker_report_for_workflow /
//   evidence_refs_for_c5 同构·不自造第二套存取）；**不收前端转述**（前端只传定位键 workflow_id+chain_started_at）；
// - 唯一写入 = `global_supervisor_review_store`（sidecar+内嵌审计）——workflow state 文件零触碰；
// - **幂等防重烧**：同 (workflow_id, chain_started_at) 已有记录（含 unavailable）→ 直接返回不 consult；
//   force=true（[重新复核]/[重试]）才重跑；
// - **任何失败不 Err 断面板**：返回结构带 status="unavailable"+人话 reason（供给类经 fix8 前缀人话透传）。

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::global_supervisor_review_store::{
    self, GlobalSupervisorReviewRecord, GlobalSupervisorTaskVerdict,
};

/// §10-1 换脑可定位：档案版本随记录落盘（档案文本变更时手动 bump）。
pub(crate) const GLOBAL_SUPERVISOR_PROFILE_VERSION: &str = "global-supervisor-profile.v1";
/// consult 走 codex CLI 默认模型（runner 不指定 model）；记录里老实写「CLI 默认」而非编一个具体名。
pub(crate) const GLOBAL_SUPERVISOR_MODEL_LABEL: &str = "codex-cli-default";

/// 档案（prompt 头）：角色=全局主管（复核最终结果·不是审批）。外部文件化是认下的债（归 Phase D），
/// 暂随 consultant/director 先例进代码。
const GLOBAL_SUPERVISOR_PROFILE_TEXT: &str = "你是本项目的「全局主管」，职责是**复核最终结果**——读用户批过的方案、本轮每个任务的执行态与 worker 口供（自报「做了什么/产出/成败/证据」），给出一份人话复核意见。你不是审批者：你的意见不拦任何事，最终判断和动作都在用户手里。要求：\n- 每个任务给一句点评；口供 status 不是 done、或没交口供的任务（黄牌）**必须点评**，说清你看到的问题；\n- 保守判：证据不足、拿不准就说拿不准（overall 用 needs_human_check），不装确定、不脑补没提供的事实；\n- 对照方案目标核「说做了的」与「方案要的」是否对得上；证据只当 worker 的自述看待，别当已核实的真相；\n- 全中文、人话、简短，别用内部黑话。";

/// 回程契约段（确定性文本·照 WORKER_REPORT_CONTRACT_TEXT 风格）。
const GLOBAL_SUPERVISOR_CONTRACT_TEXT: &str = "回程契约（务必遵守）：最后输出**且仅输出**一个 ```json 代码块，严格形如 {\"overall\":\"pass|needs_rework|needs_human_check\",\"tasks\":[{\"title\":\"任务名\",\"verdict\":\"ok|issue\",\"comment\":\"一句点评\"}],\"summary\":\"总评一两句\",\"suggested_action\":\"none|replan|human_verify\",\"human_note\":\"若建议亲验，用一句话告诉用户该亲手验什么；否则留空\"}。不要在这个 json 块之后再写任何字。";

/// LM 输出投影（serde 全 default 软着陆：缺字段不报错）。
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct SupervisorReviewJson {
    #[serde(default)]
    pub(crate) overall: String,
    #[serde(default)]
    pub(crate) tasks: Vec<SupervisorTaskJson>,
    #[serde(default)]
    pub(crate) summary: String,
    #[serde(default)]
    pub(crate) suggested_action: String,
    #[serde(default)]
    pub(crate) human_note: String,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct SupervisorTaskJson {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) verdict: String,
    #[serde(default)]
    pub(crate) comment: String,
}

/// 从主管最后消息抠出并解析意见 json 块（复用现成抠取器；抠不到/坏 json → None 软着陆）。
pub(crate) fn parse_supervisor_review(raw: &str) -> Option<SupervisorReviewJson> {
    let block = crate::consultant_extract_json_block(raw)?;
    serde_json::from_str::<SupervisorReviewJson>(&block).ok()
}

/// 词表归一化（保守向）：overall 未知 → needs_human_check（拿不准当拿不准）；
/// suggested_action 未知 → none（不给错按钮）；verdict 未知 → issue（宁可多看一眼）。
pub(crate) fn normalize_overall(raw: &str) -> String {
    match raw.trim() {
        "pass" | "needs_rework" | "needs_human_check" => raw.trim().to_string(),
        _ => "needs_human_check".to_string(),
    }
}

pub(crate) fn normalize_suggested_action(raw: &str) -> String {
    match raw.trim() {
        "none" | "replan" | "human_verify" => raw.trim().to_string(),
        _ => "none".to_string(),
    }
}

pub(crate) fn normalize_verdict(raw: &str) -> String {
    match raw.trim() {
        "ok" | "issue" => raw.trim().to_string(),
        _ => "issue".to_string(),
    }
}

// ===== 读盘组输入（全现成只读口·§0.3 核查过） =====

/// 一份口供的投影（audit_events 里 worker_structured_report_recorded 事件的字段子集）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkerReportProjection {
    pub(crate) work_item_id: String,
    pub(crate) executed_what: String,
    pub(crate) changed_what: String,
    pub(crate) acceptance_status: String,
    pub(crate) evidence_refs: Vec<String>,
    pub(crate) open_issues: Vec<String>,
}

/// 复核输入（全从盘读）。
#[derive(Debug, Clone, Default)]
pub(crate) struct SupervisorReviewInput {
    pub(crate) proposal_title: Option<String>,
    pub(crate) proposal_goal: Option<String>,
    pub(crate) proposal_steps: Vec<String>,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) chain_state: String,
    pub(crate) chain_started_at: String,
    pub(crate) chain_ended_at: Option<String>,
    pub(crate) chain_nodes: Vec<(String, String)>, // (node_id, state) 角色节点执行态
    pub(crate) task_nodes: Vec<(String, String)>,  // (node_id, state) 任务级节点
    pub(crate) reports: Vec<WorkerReportProjection>,
}

/// 防 prompt 爆炸：单字段截断（按 char 边界安全）。
fn clip(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= max_chars {
        trimmed.to_string()
    } else {
        let head: String = chars[..max_chars].iter().collect();
        format!("{head}…（截断）")
    }
}

/// 读盘组输入：链轮按 (workflow_id, started_at==chain_started_at) 精确匹配（不用「最新一条」，
/// 防复核期间又起新链串轮）；口供按 (workflow_id + created_at ∈ [started, ended||∞]) 圈本轮
/// （两者同源 unix 毫秒字符串·可 parse 比较）。链轮找不到 → Err（调用方软着陆成 unavailable）。
pub(crate) fn load_review_input(
    state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    chain_started_at: &str,
) -> Result<SupervisorReviewInput, String> {
    let value = crate::read_workflow_state_value(state_path)?;
    let pid = crate::project_id(project_root);
    // 1. 本轮链记录（精确匹配）。
    let run = value
        .get("workflow_chain_runs")
        .and_then(serde_json::Value::as_array)
        .and_then(|runs| {
            runs.iter().find(|run| {
                crate::optional_string_from(run, "workflow_id").as_deref() == Some(workflow_id)
                    && crate::optional_string_from(run, "project_id").as_deref()
                        == Some(pid.as_str())
                    && crate::optional_string_from(run, "started_at").as_deref()
                        == Some(chain_started_at)
            })
        })
        .ok_or_else(|| "没找到这一轮的链记录（可能传错轮次，或这轮还没起链）".to_string())?;
    let chain_state = crate::optional_string_from(run, "state").unwrap_or_default();
    let chain_ended_at = crate::optional_string_from(run, "ended_at");
    let chain_nodes = run
        .get("nodes")
        .and_then(serde_json::Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .map(|node| {
                    (
                        crate::optional_string_from(node, "node_id").unwrap_or_default(),
                        crate::optional_string_from(node, "state").unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // 2. 本轮口供（时间窗：started ≤ created_at ≤ ended（没收尾则到现在））。
    let window_start = chain_started_at.parse::<i64>().unwrap_or(0);
    let window_end = chain_ended_at
        .as_deref()
        .and_then(|end| end.parse::<i64>().ok())
        .unwrap_or(i64::MAX);
    let reports = value
        .get("audit_events")
        .and_then(serde_json::Value::as_array)
        .map(|events| {
            events
                .iter()
                .filter(|event| {
                    crate::optional_string_from(event, "event_type").as_deref()
                        == Some("worker_structured_report_recorded")
                        && crate::optional_string_from(event, "workflow_id").as_deref()
                            == Some(workflow_id)
                        && crate::optional_string_from(event, "created_at")
                            .and_then(|created| created.parse::<i64>().ok())
                            .map(|created| created >= window_start && created <= window_end)
                            .unwrap_or(false)
                })
                .map(|event| WorkerReportProjection {
                    work_item_id: crate::optional_string_from(event, "work_item_id")
                        .unwrap_or_default(),
                    executed_what: crate::optional_string_from(event, "executed_what")
                        .unwrap_or_default(),
                    changed_what: crate::optional_string_from(event, "changed_what")
                        .unwrap_or_default(),
                    acceptance_status: crate::optional_string_from(event, "acceptance_status")
                        .unwrap_or_default(),
                    evidence_refs: string_vec(event, "evidence_refs"),
                    open_issues: string_vec(event, "open_issues"),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // 3. 任务级节点执行态（刀2 落画布的 `{wf}:node:task:` 前缀）。
    let task_prefix = format!("{workflow_id}:node:task:");
    let task_nodes = value
        .get("nodes")
        .and_then(serde_json::Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter(|node| {
                    crate::optional_string_from(node, "node_id")
                        .map(|node_id| node_id.starts_with(&task_prefix))
                        .unwrap_or(false)
                })
                .map(|node| {
                    (
                        crate::optional_string_from(node, "node_id").unwrap_or_default(),
                        crate::optional_string_from(node, "state").unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // 4. 所批方案（该 workflow 最新一条已确认；找不到不拦——prompt 里注明）。
    let timestamp_ms = crate::unix_timestamp_ms();
    let (proposal_title, proposal_goal, proposal_steps, allowed_write_roots) =
        match crate::project_consultation_proposal_store::load_store(state_path, timestamp_ms) {
            Ok(store) => {
                let confirmed = store
                    .proposals
                    .iter()
                    .filter(|proposal| {
                        proposal.workflow_id == workflow_id
                            && matches!(
                                proposal.status,
                                crate::ProjectConsultationProposalStatus::UserConfirmed
                            )
                    })
                    .max_by_key(|proposal| proposal.created_at_ms);
                match confirmed {
                    Some(proposal) => (
                        Some(proposal.title.clone()),
                        Some(proposal.goal_summary.clone()),
                        proposal.proposed_steps.clone(),
                        proposal.scope_draft.allowed_write_roots.clone(),
                    ),
                    None => (None, None, Vec::new(), Vec::new()),
                }
            }
            Err(_) => (None, None, Vec::new(), Vec::new()),
        };
    Ok(SupervisorReviewInput {
        proposal_title,
        proposal_goal,
        proposal_steps,
        allowed_write_roots,
        chain_state,
        chain_started_at: chain_started_at.to_string(),
        chain_ended_at,
        chain_nodes,
        task_nodes,
        reports,
    })
}

fn string_vec(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// 组 prompt（档案 + 盘上输入 + 契约段·确定性拼接不经 LM）。
pub(crate) fn build_supervisor_prompt(input: &SupervisorReviewInput) -> String {
    let mut sections: Vec<String> = vec![GLOBAL_SUPERVISOR_PROFILE_TEXT.to_string()];
    match (&input.proposal_title, &input.proposal_goal) {
        (Some(title), goal) => {
            let mut block = format!("【用户批的方案】{}", clip(title, 200));
            if let Some(goal) = goal {
                block.push_str(&format!("\n目标：{}", clip(goal, 600)));
            }
            if !input.proposal_steps.is_empty() {
                block.push_str("\n步骤：");
                for (index, step) in input.proposal_steps.iter().take(12).enumerate() {
                    block.push_str(&format!("\n{}. {}", index + 1, clip(step, 300)));
                }
            }
            if !input.allowed_write_roots.is_empty() {
                block.push_str(&format!(
                    "\n允许改动范围：{}",
                    input.allowed_write_roots.join("；")
                ));
            }
            sections.push(block);
        }
        _ => sections.push(
            "【用户批的方案】盘上没找到这轮对应的已确认方案（照常复核，但请在总评注明缺方案对照）。"
                .to_string(),
        ),
    }
    let ended = input.chain_ended_at.as_deref().unwrap_or("（未收尾）");
    let mut chain_block = format!(
        "【本轮链】状态 {}；起 {}；止 {}。角色节点执行态：",
        input.chain_state, input.chain_started_at, ended
    );
    if input.chain_nodes.is_empty() {
        chain_block.push_str("（无）");
    } else {
        for (node_id, state) in &input.chain_nodes {
            chain_block.push_str(&format!("\n- {node_id}：{state}"));
        }
    }
    sections.push(chain_block);
    let mut tasks_block = format!(
        "【任务节点】共 {} 个（本轮收到 {} 份口供；数对不上就是有任务没交口供=黄牌）：",
        input.task_nodes.len(),
        input.reports.len()
    );
    if input.task_nodes.is_empty() {
        tasks_block.push_str("（无任务级节点·可能是简单单）");
    } else {
        for (node_id, state) in &input.task_nodes {
            tasks_block.push_str(&format!("\n- {node_id}：{state}"));
        }
    }
    sections.push(tasks_block);
    if input.reports.is_empty() {
        sections.push("【每任务口供】本轮一份口供都没有——这本身就该在意见里说清。".to_string());
    } else {
        let mut reports_block = "【每任务口供】（worker 自报·未核实）".to_string();
        for (index, report) in input.reports.iter().enumerate() {
            reports_block.push_str(&format!(
                "\n{}. 工作项 {}：做了「{}」；产出「{}」；自报状态 {}；证据：{}；遗留问题：{}",
                index + 1,
                report.work_item_id,
                clip(&report.executed_what, 400),
                clip(&report.changed_what, 300),
                if report.acceptance_status.trim().is_empty() {
                    "（未报）"
                } else {
                    report.acceptance_status.as_str()
                },
                if report.evidence_refs.is_empty() {
                    "（没给）".to_string()
                } else {
                    report
                        .evidence_refs
                        .iter()
                        .map(|evidence| clip(evidence, 200))
                        .collect::<Vec<_>>()
                        .join("；")
                },
                if report.open_issues.is_empty() {
                    "（无）".to_string()
                } else {
                    report
                        .open_issues
                        .iter()
                        .map(|issue| clip(issue, 200))
                        .collect::<Vec<_>>()
                        .join("；")
                },
            ));
        }
        sections.push(reports_block);
    }
    sections.push(GLOBAL_SUPERVISOR_CONTRACT_TEXT.to_string());
    sections.join("\n\n")
}

// ===== 命令核心（consult 可注入·单测 stub 计次） =====

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RunGlobalSupervisorReviewRequest {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    /// 前端只传定位键（哪一轮），不传内容（输入全从盘读·不收转述）。
    pub(crate) chain_started_at: String,
    #[serde(default)]
    pub(crate) force: bool,
}

/// 返回结构（**任何失败不 Err 断面板**——status 带态）：
/// - "ready"：意见在 review 里；
/// - "unavailable"：没跑成，reason 是人话（供给类已带 fix8 人话），可 [重试]（force）。
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GlobalSupervisorReviewOutcome {
    pub(crate) status: String,
    pub(crate) review: Option<GlobalSupervisorReviewRecord>,
    pub(crate) reason: Option<String>,
    pub(crate) warnings: Vec<String>,
}

/// 把 consult 错误翻成人话（fix8 供给类前缀直接取其人话；其余原样带出）。
fn humanize_consult_error(raw: &str) -> String {
    match raw.strip_prefix("codex_provider_unavailable:") {
        Some(human) => human.trim().to_string(),
        None => raw.to_string(),
    }
}

/// 核心（同步·可注入 consult 供单测 stub 计次）。
/// 幂等（成本护栏）：同 (workflow_id, chain_started_at) 已有记录（**含 unavailable**）且 !force
/// → 直接返回既有记录、不 consult；[重试]/[重新复核] 走 force=true。
pub(crate) fn run_global_supervisor_review_core<F>(
    state_path: &Path,
    request: &RunGlobalSupervisorReviewRequest,
    consult: F,
) -> GlobalSupervisorReviewOutcome
where
    F: Fn(&str, &str) -> Result<String, String>,
{
    let timestamp_ms = crate::unix_timestamp_ms();
    let mut warnings: Vec<String> = Vec::new();
    if request.workflow_id.trim().is_empty() || request.chain_started_at.trim().is_empty() {
        return GlobalSupervisorReviewOutcome {
            status: "unavailable".to_string(),
            review: None,
            reason: Some("缺 workflow_id / chain_started_at（不知道要复核哪一轮）".to_string()),
            warnings,
        };
    }
    // 1. 幂等命中：已有记录且非 force → 原样返回（不 consult·不写盘）。
    let (store, load_warnings) =
        global_supervisor_review_store::load_store_soft(state_path, timestamp_ms);
    warnings.extend(load_warnings);
    if !request.force {
        if let Some(existing) = global_supervisor_review_store::find_review(
            &store,
            &request.workflow_id,
            &request.chain_started_at,
        ) {
            return GlobalSupervisorReviewOutcome {
                status: existing.status.clone(),
                reason: existing.unavailable_reason.clone(),
                review: Some(existing.clone()),
                warnings,
            };
        }
    }
    // 2. 读盘组输入（链轮找不到 → unavailable、不落盘：键不对落了也是垃圾）。
    let input = match load_review_input(
        state_path,
        &request.project_root,
        &request.workflow_id,
        &request.chain_started_at,
    ) {
        Ok(input) => input,
        Err(error) => {
            return GlobalSupervisorReviewOutcome {
                status: "unavailable".to_string(),
                review: None,
                reason: Some(format!("复核不可用：{error}")),
                warnings,
            };
        }
    };
    let prompt = build_supervisor_prompt(&input);
    // 3. consult（只读）→ 解析 → 归一化；失败落「复核不可用」记录（可重试）。
    let base_record = GlobalSupervisorReviewRecord {
        review_id: format!(
            "global-supervisor-review:{}:{}",
            request.workflow_id, request.chain_started_at
        ),
        project_id: crate::project_id(&request.project_root),
        workflow_id: request.workflow_id.clone(),
        chain_started_at: request.chain_started_at.clone(),
        model: GLOBAL_SUPERVISOR_MODEL_LABEL.to_string(),
        profile_version: GLOBAL_SUPERVISOR_PROFILE_VERSION.to_string(),
        ..Default::default()
    };
    let record = match consult(&request.project_root, &prompt) {
        Ok(raw) => match parse_supervisor_review(&raw) {
            Some(parsed) => {
                let human_note = parsed.human_note.trim().to_string();
                let suggested_action = normalize_suggested_action(&parsed.suggested_action);
                GlobalSupervisorReviewRecord {
                    status: "ready".to_string(),
                    overall: normalize_overall(&parsed.overall),
                    summary: parsed.summary.trim().to_string(),
                    human_note: if suggested_action == "human_verify" && human_note.is_empty() {
                        "建议你亲自核验这轮结果。".to_string()
                    } else {
                        human_note
                    },
                    suggested_action,
                    tasks: parsed
                        .tasks
                        .into_iter()
                        .map(|task| GlobalSupervisorTaskVerdict {
                            title: task.title.trim().to_string(),
                            verdict: normalize_verdict(&task.verdict),
                            comment: task.comment.trim().to_string(),
                        })
                        .collect(),
                    unavailable_reason: None,
                    ..base_record.clone()
                }
            }
            None => GlobalSupervisorReviewRecord {
                status: "unavailable".to_string(),
                unavailable_reason: Some("全局主管没按契约交回意见 json（可重试）".to_string()),
                ..base_record.clone()
            },
        },
        Err(error) => GlobalSupervisorReviewRecord {
            status: "unavailable".to_string(),
            unavailable_reason: Some(humanize_consult_error(&error)),
            ..base_record.clone()
        },
    };
    // 4. 落库（唯一写入面）。写失败也不 Err 断面板——意见还在返回体里，只是没存住。
    match global_supervisor_review_store::upsert_review(
        state_path,
        record.clone(),
        "global_supervisor_agent",
        timestamp_ms,
    ) {
        Ok(_) => {}
        Err(error) => warnings.push(format!("复核意见落库失败（意见仍返回）：{error}")),
    }
    GlobalSupervisorReviewOutcome {
        status: record.status.clone(),
        reason: record.unavailable_reason.clone(),
        review: Some(record),
        warnings,
    }
}

/// consult 超时：复核输入比咨询小（口供投影），但 tier-1 长思考常见 → 沿用 consultant 家族 420s。
const GLOBAL_SUPERVISOR_CONSULT_TIMEOUT_MS: i64 = 420_000;

#[tauri::command]
pub(crate) async fn run_global_supervisor_review(
    request: RunGlobalSupervisorReviewRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<GlobalSupervisorReviewOutcome, String> {
    // 真 consult 长耗时 → spawn_blocking 不冻 UI（同合流/咨询范本）；path 在 await 前取。
    let path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_global_supervisor_review_core(&path, &request, |project_root, prompt| {
            crate::codex_local_runner::readonly_codex_consult(
                project_root,
                prompt,
                Some(GLOBAL_SUPERVISOR_CONSULT_TIMEOUT_MS),
            )
        })
    })
    .await
    .map_err(|error| format!("复核执行线程异常：{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_dir(tag: &str) -> PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("gsr-agent-{tag}-{uniq}"));
        fs::create_dir_all(&dir).expect("tmp dir");
        dir
    }

    /// 最小 fixture store：一轮链（started 1000/ended 2000）+ 窗口内 2 份口供（1 done + 1 partial）
    /// + 窗口外 1 份（不该被圈进）+ 2 个任务节点。自包含手写、不依赖 lib.rs helper（worker_report 先例）。
    fn write_fixture_state(dir: &Path, project_root: &str) -> PathBuf {
        let pid = crate::project_id(project_root);
        let store = serde_json::json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": "seed",
            "projects": [],
            "agent_adapters": [],
            "workflows": [{"workflow_id": "wf-1"}],
            "nodes": [
                {"workflow_id": "wf-1", "node_id": "wf-1:node:codex-dev", "state": "completed"},
                {"workflow_id": "wf-1", "node_id": "wf-1:node:task:aaa", "state": "completed"},
                {"workflow_id": "wf-1", "node_id": "wf-1:node:task:bbb", "state": "completed"}
            ],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "audit_events": [
                {
                    "event_id": "r1", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-1", "created_at": "1500",
                    "executed_what": "建了 index.html 小游戏", "changed_what": "/t/index.html",
                    "acceptance_status": "reported_completed",
                    "evidence_refs": ["文件存在且能打开"], "open_issues": []
                },
                {
                    "event_id": "r2", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-2", "created_at": "1800",
                    "executed_what": "无法启动浏览器，未完成手动验收", "changed_what": "（无产出）",
                    "acceptance_status": "needs_rework",
                    "evidence_refs": [], "open_issues": ["浏览器起不来"]
                },
                {
                    "event_id": "r-out", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-old", "created_at": "500",
                    "executed_what": "上一轮的旧口供（不该被圈进）", "changed_what": "x",
                    "acceptance_status": "reported_completed",
                    "evidence_refs": [], "open_issues": []
                }
            ],
            "capabilities": [],
            "harness_resources": [],
            "workflow_chain_runs": [{
                "chain_run_id": "chain-1", "project_id": pid, "workflow_id": "wf-1",
                "state": "completed", "stop_requested": false,
                "started_at": "1000", "ended_at": "2000",
                "nodes": [{"node_id": "wf-1:node:codex-dev", "state": "completed", "dispatch_id": null, "message": null}]
            }]
        });
        let path = dir.join("workflow-state.v0.json");
        fs::write(&path, serde_json::to_string_pretty(&store).unwrap()).expect("write state");
        path
    }

    const GOOD_REVIEW: &str = "看完了。\n```json\n{\"overall\":\"needs_human_check\",\"tasks\":[{\"title\":\"建小游戏\",\"verdict\":\"ok\",\"comment\":\"口供与产出对得上\"},{\"title\":\"手动验收\",\"verdict\":\"issue\",\"comment\":\"worker 自报浏览器起不来、没完成验收\"}],\"summary\":\"一单没验收完，建议亲验\",\"suggested_action\":\"human_verify\",\"human_note\":\"打开 index.html 亲手玩一遍\"}\n```";

    // §4·schema 三态①：合法块解析 + 归一化透传。
    #[test]
    fn parses_valid_review_block() {
        let parsed = parse_supervisor_review(GOOD_REVIEW).expect("合法块应解析");
        assert_eq!(parsed.overall, "needs_human_check");
        assert_eq!(parsed.tasks.len(), 2);
        assert_eq!(parsed.suggested_action, "human_verify");
    }

    // §4·schema 三态②：缺字段 default 容忍（不 Err）+ 归一化保守向。
    #[test]
    fn missing_fields_default_and_normalize_conservative() {
        let parsed = parse_supervisor_review("```json\n{\"summary\":\"只给了总评\"}\n```")
            .expect("缺字段也解析");
        assert_eq!(parsed.summary, "只给了总评");
        assert!(parsed.tasks.is_empty());
        // 归一化：overall 空/未知 → needs_human_check；action 未知 → none；verdict 未知 → issue。
        assert_eq!(normalize_overall(""), "needs_human_check");
        assert_eq!(normalize_overall("approved"), "needs_human_check");
        assert_eq!(normalize_overall("pass"), "pass");
        assert_eq!(normalize_suggested_action("do_magic"), "none");
        assert_eq!(normalize_suggested_action("replan"), "replan");
        assert_eq!(normalize_verdict("great"), "issue");
        assert_eq!(normalize_verdict("ok"), "ok");
    }

    // §4·schema 三态③：坏 json / 无块 → None（软着陆·由 core 落「不可用」记录）。
    #[test]
    fn broken_or_missing_block_is_none() {
        assert!(parse_supervisor_review("没有块").is_none());
        assert!(parse_supervisor_review("```json\n{坏的\n```").is_none());
    }

    // 读盘：链轮精确匹配 + 口供按时间窗圈本轮（窗外旧口供不进）+ 任务节点圈对。
    #[test]
    fn load_input_scopes_reports_to_this_round() {
        let dir = tmp_dir("scope");
        let path = write_fixture_state(&dir, "/p/root");
        let input = load_review_input(&path, "/p/root", "wf-1", "1000").expect("输入应组出来");
        assert_eq!(input.chain_state, "completed");
        assert_eq!(
            input.reports.len(),
            2,
            "窗内 2 份（窗外 created_at=500 不进）"
        );
        assert!(input
            .reports
            .iter()
            .all(|report| report.work_item_id != "wi-old"));
        assert_eq!(input.task_nodes.len(), 2, "任务节点按 :node:task: 前缀圈");
        // 链轮找不到 → Err 人话。
        let err = load_review_input(&path, "/p/root", "wf-1", "9999").expect_err("错轮应报");
        assert!(err.contains("没找到这一轮"), "人话：{err}");
        // prompt 组装含关键素材与契约。
        let prompt = build_supervisor_prompt(&input);
        assert!(prompt.contains("无法启动浏览器"), "口供进 prompt");
        assert!(
            prompt.contains("共 2 个") && prompt.contains("2 份口供"),
            "任务/口供计数进 prompt"
        );
        assert!(prompt.contains("回程契约"), "契约段在");
        let _ = fs::remove_dir_all(dir);
    }

    // §4：幂等命中不重跑（stub consult 计次=1）+ force 重跑（=2）。
    #[test]
    fn idempotent_hit_skips_consult_and_force_reruns() {
        let dir = tmp_dir("idem");
        let path = write_fixture_state(&dir, "/p/root");
        let calls = Cell::new(0usize);
        let consult = |_root: &str, _prompt: &str| {
            calls.set(calls.get() + 1);
            Ok(GOOD_REVIEW.to_string())
        };
        let request = RunGlobalSupervisorReviewRequest {
            project_root: "/p/root".to_string(),
            workflow_id: "wf-1".to_string(),
            chain_started_at: "1000".to_string(),
            force: false,
        };
        let first = run_global_supervisor_review_core(&path, &request, consult);
        assert_eq!(first.status, "ready", "{:?}", first.reason);
        assert_eq!(calls.get(), 1);
        let review = first.review.expect("ready 应带记录");
        assert_eq!(review.overall, "needs_human_check");
        assert_eq!(review.suggested_action, "human_verify");
        assert_eq!(review.tasks.len(), 2);
        assert_eq!(
            review.model, GLOBAL_SUPERVISOR_MODEL_LABEL,
            "§10-1 model 落记录"
        );
        assert_eq!(review.profile_version, GLOBAL_SUPERVISOR_PROFILE_VERSION);
        // 第二次（非 force）：幂等命中，consult 不再被调。
        let second = run_global_supervisor_review_core(&path, &request, consult);
        assert_eq!(second.status, "ready");
        assert_eq!(calls.get(), 1, "幂等命中不得重烧 consult");
        // force：重跑。
        let forced = RunGlobalSupervisorReviewRequest {
            force: true,
            ..request.clone()
        };
        let third = run_global_supervisor_review_core(&path, &forced, consult);
        assert_eq!(third.status, "ready");
        assert_eq!(calls.get(), 2, "force 才重跑");
        let _ = fs::remove_dir_all(dir);
    }

    // §4：供给类失败 → 人话（剥 codex_provider_unavailable: 前缀）+ 落「不可用」记录可重试；
    // unavailable 记录也参与幂等（自动路不重烧），force 重试后翻 ready。
    #[test]
    fn provider_failure_humanized_and_retryable() {
        let dir = tmp_dir("provider");
        let path = write_fixture_state(&dir, "/p/root");
        let calls = Cell::new(0usize);
        let failing = |_root: &str, _prompt: &str| {
            calls.set(calls.get() + 1);
            Err("codex_provider_unavailable:codex 额度用完了，明天再试或升级订阅".to_string())
        };
        let request = RunGlobalSupervisorReviewRequest {
            project_root: "/p/root".to_string(),
            workflow_id: "wf-1".to_string(),
            chain_started_at: "1000".to_string(),
            force: false,
        };
        let outcome = run_global_supervisor_review_core(&path, &request, failing);
        assert_eq!(outcome.status, "unavailable");
        let reason = outcome.reason.clone().expect("应带人话原因");
        assert!(reason.contains("额度用完"), "供给类人话直取：{reason}");
        assert!(
            !reason.contains("codex_provider_unavailable:"),
            "前缀应剥掉"
        );
        // 记录落盘（可重试）。
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_000);
        let saved = global_supervisor_review_store::find_review(&store, "wf-1", "1000")
            .expect("不可用也落记录");
        assert_eq!(saved.status, "unavailable");
        // 非 force 再来：幂等命中（不重烧）。
        let again = run_global_supervisor_review_core(&path, &request, failing);
        assert_eq!(again.status, "unavailable");
        assert_eq!(calls.get(), 1, "unavailable 也参与幂等·自动路不重烧");
        // force + 供给恢复 → ready 覆盖同键记录。
        let recovered = |_root: &str, _prompt: &str| Ok(GOOD_REVIEW.to_string());
        let forced = RunGlobalSupervisorReviewRequest {
            force: true,
            ..request.clone()
        };
        let retried = run_global_supervisor_review_core(&path, &forced, recovered);
        assert_eq!(retried.status, "ready");
        let (store2, _) = global_supervisor_review_store::load_store_soft(&path, 9_999);
        assert_eq!(store2.reviews.len(), 1, "同键覆盖不追加");
        assert_eq!(store2.reviews[0].status, "ready");
        let _ = fs::remove_dir_all(dir);
    }

    // 坏回包（不按契约）→ 落「不可用」记录 + 人话；链轮不存在 → unavailable 但不落盘。
    #[test]
    fn bad_reply_recorded_and_missing_round_not_recorded() {
        let dir = tmp_dir("bad");
        let path = write_fixture_state(&dir, "/p/root");
        let no_contract =
            |_root: &str, _prompt: &str| Ok("我看完了，都挺好（没给 json）".to_string());
        let request = RunGlobalSupervisorReviewRequest {
            project_root: "/p/root".to_string(),
            workflow_id: "wf-1".to_string(),
            chain_started_at: "1000".to_string(),
            force: false,
        };
        let outcome = run_global_supervisor_review_core(&path, &request, no_contract);
        assert_eq!(outcome.status, "unavailable");
        assert!(outcome.reason.clone().unwrap_or_default().contains("契约"));
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_000);
        assert!(global_supervisor_review_store::find_review(&store, "wf-1", "1000").is_some());
        // 链轮不存在：unavailable + 不落盘（键不对落了也是垃圾）。
        let missing = RunGlobalSupervisorReviewRequest {
            project_root: "/p/root".to_string(),
            workflow_id: "wf-1".to_string(),
            chain_started_at: "424242".to_string(),
            force: false,
        };
        let outcome2 = run_global_supervisor_review_core(&path, &missing, no_contract);
        assert_eq!(outcome2.status, "unavailable");
        assert!(outcome2.reason.unwrap_or_default().contains("没找到这一轮"));
        let (store2, _) = global_supervisor_review_store::load_store_soft(&path, 9_100);
        assert!(
            global_supervisor_review_store::find_review(&store2, "wf-1", "424242").is_none(),
            "错轮不落盘"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4·真跑（单独步·#[ignore]·固定测试项目·额度在）：对最近一轮真链跑真复核。
    // 显式 `cargo test --lib global_supervisor_review_real_run -- --ignored --nocapture`。
    // 核实物：意见 grounded（打印全文供主导线人核）·记录落盘·store 内嵌审计在·`.codex` auth 未碰。
    #[test]
    #[ignore = "B1 global supervisor: real read-only review of the latest real chain round in the test project (user present, quota available)"]
    fn global_supervisor_review_real_run() {
        let state_path = crate::default_workflow_state_path();
        assert!(
            state_path.exists(),
            "真 store 应存在：{}",
            state_path.display()
        );
        let auth_path = PathBuf::from(std::env::var("HOME").expect("HOME"))
            .join(".codex")
            .join("auth.json");
        let auth_before = fs::metadata(&auth_path)
            .and_then(|meta| meta.modified())
            .ok();
        // 找最近一轮已收尾真链（不猜 workflow——从盘上真数据取）。
        let value = crate::read_workflow_state_value(&state_path).expect("read state");
        let run = value
            .get("workflow_chain_runs")
            .and_then(serde_json::Value::as_array)
            .and_then(|runs| {
                runs.iter()
                    .filter(|run| crate::optional_string_from(run, "ended_at").is_some())
                    .max_by_key(|run| {
                        crate::optional_string_from(run, "started_at")
                            .and_then(|started| started.parse::<i64>().ok())
                            .unwrap_or(0)
                    })
            })
            .expect("盘上应有至少一轮已收尾链（先真机跑一单交办）")
            .clone();
        let workflow_id =
            crate::optional_string_from(&run, "workflow_id").expect("链记录带 workflow_id");
        let chain_started_at =
            crate::optional_string_from(&run, "started_at").expect("链记录带 started_at");
        println!("[B1_REAL] 复核轮：workflow={workflow_id} started_at={chain_started_at}");
        let request = RunGlobalSupervisorReviewRequest {
            project_root: "/Users/yoyi/codex-workflow-mario-test".to_string(),
            workflow_id: workflow_id.clone(),
            chain_started_at: chain_started_at.clone(),
            // 真跑要真 consult：force 穿透可能已存在的记录（此前真机可能已自动复核过）。
            force: true,
        };
        let outcome = run_global_supervisor_review_core(&state_path, &request, |root, prompt| {
            crate::codex_local_runner::readonly_codex_consult(root, prompt, Some(420_000))
        });
        println!(
            "[B1_REAL] status={} reason={:?}",
            outcome.status, outcome.reason
        );
        let review = outcome.review.clone().expect("应带记录");
        println!(
            "[B1_REAL] overall={} action={} summary={}",
            review.overall, review.suggested_action, review.summary
        );
        for task in &review.tasks {
            println!(
                "[B1_REAL] - [{}] {}：{}",
                task.verdict, task.title, task.comment
            );
        }
        assert_eq!(
            outcome.status, "ready",
            "真复核应出意见：{:?}",
            outcome.reason
        );
        assert!(!review.summary.trim().is_empty(), "总评非空");
        assert!(!review.tasks.is_empty(), "应有每任务点评");
        // 记录真落盘 + 审计内嵌。
        let (store, _) = global_supervisor_review_store::load_store_soft(&state_path, 0);
        let saved =
            global_supervisor_review_store::find_review(&store, &workflow_id, &chain_started_at)
                .expect("记录应落盘");
        assert_eq!(saved.status, "ready");
        assert!(
            store
                .audit_events
                .iter()
                .any(|event| event.event_type == "global_supervisor_review_recorded"),
            "store 内嵌审计在"
        );
        // `.codex` 凭据死线。
        let auth_after = fs::metadata(&auth_path)
            .and_then(|meta| meta.modified())
            .ok();
        assert_eq!(auth_before, auth_after, ".codex 凭据不许被碰");
    }
}
