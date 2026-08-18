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
use std::collections::BTreeSet;
use std::path::Path;

use crate::global_supervisor_review_store::{
    self, GlobalSupervisorBoundaryReviewRecord, GlobalSupervisorReviewRecord,
    GlobalSupervisorTaskVerdict,
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
/// 防复核期间又起新链串轮）；口供优先按本轮 run.nodes 的 dispatch_id 精确圈定，避免收尾与口供
/// 同毫秒竞态；旧 run 记录没有可用 dispatch_id 时才以收尾后 60 秒的窄时间窗软着陆。链轮找不到
/// → Err（调用方软着陆成 unavailable）。
pub(crate) fn load_review_input(
    state_path: &Path,
    canonical_project_id: &str,
    workflow_id: &str,
    chain_started_at: &str,
) -> Result<SupervisorReviewInput, String> {
    let value = crate::read_workflow_state_value(state_path)?;
    // 1. 本轮链记录（canonical project id + workflow_id + started_at 精确匹配）。
    let run = value
        .get("workflow_chain_runs")
        .and_then(serde_json::Value::as_array)
        .and_then(|runs| {
            runs.iter().find(|run| {
                crate::optional_string_from(run, "workflow_id").as_deref() == Some(workflow_id)
                    && crate::optional_string_from(run, "project_id").as_deref()
                        == Some(canonical_project_id)
                    && crate::optional_string_from(run, "started_at").as_deref()
                        == Some(chain_started_at)
            })
        })
        .ok_or_else(|| "没找到这一轮的链记录（可能传错轮次，或这轮还没起链）".to_string())?;
    let chain_state = crate::optional_string_from(run, "state").unwrap_or_default();
    let chain_ended_at = crate::optional_string_from(run, "ended_at");
    let run_nodes = run
        .get("nodes")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let chain_nodes = run_nodes
        .iter()
        .map(|node| {
            (
                crate::optional_string_from(node, "node_id").unwrap_or_default(),
                crate::optional_string_from(node, "state").unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    // 2. 本轮口供（优先按 dispatch_id；旧 run 才走窄时间窗）。
    let window_start = chain_started_at.parse::<i64>().unwrap_or(0);
    let window_end = chain_ended_at
        .as_deref()
        .and_then(|end| end.parse::<i64>().ok())
        // 旧 run 记录未落 dispatch_id 时，用窄容差兜住停链和口供同毫秒竞态。
        // TODO: 历史 run 补齐可匹配标识后，删除这条时间窗回退。
        .map(|end| end.saturating_add(60_000))
        .unwrap_or(i64::MAX);
    let run_dispatch_ids = run_nodes
        .iter()
        .filter_map(|node| crate::optional_string_from(node, "dispatch_id"))
        .filter(|dispatch_id| !dispatch_id.trim().is_empty())
        .collect::<BTreeSet<_>>();
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
                        && if run_dispatch_ids.is_empty() {
                            crate::optional_string_from(event, "created_at")
                                .and_then(|created| created.parse::<i64>().ok())
                                .map(|created| created >= window_start && created <= window_end)
                                .unwrap_or(false)
                        } else {
                            crate::optional_string_from(event, "dispatch_id")
                                .map(|dispatch_id| run_dispatch_ids.contains(&dispatch_id))
                                .unwrap_or(false)
                        }
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
    // 3. 任务级节点执行态只投影本轮 run.nodes，不能让全局节点的旧轮状态混进复核。
    let task_prefix = format!("{workflow_id}:node:task:");
    let task_nodes = run_nodes
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
        .collect::<Vec<_>>();
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

/// consult 错误翻人话走单一真源 `run_error_translation`——A·收编（2026-07-09，原逐字节重复的
/// humanize 已删·供给前缀语义不变、非供给错误现也翻人话）。人话工程①②(2026-07-20)删薄委托壳,
/// 调用点直调 humanize_error_for_display。不动 director retry 读法。

/// 核心（同步·可注入 consult 供单测 stub 计次）。
/// 幂等（成本护栏）：同 (workflow_id, chain_started_at) 已有记录（**含 unavailable**）且 !force
/// → 直接返回既有记录、不 consult；[重试]/[重新复核] 走 force=true。
pub(crate) fn run_global_supervisor_review_core<F>(
    state_path: &Path,
    request: &RunGlobalSupervisorReviewRequest,
    canonical_project_id: &str,
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
    // 1. 幂等命中：已有 canonical 记录且非 force → 原样返回（不 consult·不写盘）。
    // 同键 foreign 记录不得返回、覆盖或泄露。
    let (store, load_warnings) =
        global_supervisor_review_store::load_store_soft(state_path, timestamp_ms);
    warnings.extend(load_warnings);
    if let Some(blocked) = reject_foreign_same_key_review(
        &store,
        &request.workflow_id,
        &request.chain_started_at,
        canonical_project_id,
    ) {
        return blocked;
    }
    if !request.force {
        if let Some(existing) = find_canonical_review(
            &store,
            &request.workflow_id,
            &request.chain_started_at,
            canonical_project_id,
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
        canonical_project_id,
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
        project_id: canonical_project_id.to_string(),
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
            unavailable_reason: Some(crate::run_error_translation::humanize_error_for_display(
                &error,
            )),
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
    let canonical_project_id = resolve_global_supervisor_canonical_project_id(
        state.m1_project_index_read_port(),
        &request.project_root,
    )?;
    // 真 consult 长耗时 → spawn_blocking 不冻 UI（同合流/咨询范本）；path 在 await 前取。
    let path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_global_supervisor_review_core(
            &path,
            &request,
            &canonical_project_id,
            |project_root, prompt| {
                crate::codex_local_runner::readonly_codex_consult(
                    project_root,
                    prompt,
                    Some(GLOBAL_SUPERVISOR_CONSULT_TIMEOUT_MS),
                )
            },
        )
    })
    .await
    .map_err(|error| format!("复核执行线程异常：{error}"))
}

// ============================================================================
// B2·全局主管·批前边界意见（authorize card 上「这方案对不对得上你的目标」）。
//
// 任务包：tasks/2026-07-07-phase-b2-boundary-opinion-on-authorize-card-v1.md
//
// 与 B1（结果复核·跑后·读口供）同族、同 harness（readonly consult / extract_json_block /
// project_id / unix_timestamp_ms 全复用），只是钩点不同：**批前**读盘上 pending 方案，出一句
// 「范围/目标对不对得上」的人话意见，上授权卡。
//
// 安全属性照 B1 同款：意见不是闸（不拦批·不驱动状态·词表禁「审批」）；结构性只读（readonly consult）；
// 唯一写 = store 的 boundary_reviews + 内嵌审计；幂等 by proposal_id（含 unavailable·防重烧）；
// 任何失败不 Err 断面板（status="unavailable" + 人话）。
// ============================================================================

/// B2 档案版本（独立于 B1 结果复核档案·档案文本变更时手动 bump）。
pub(crate) const GLOBAL_SUPERVISOR_BOUNDARY_PROFILE_VERSION: &str =
    "global-supervisor-boundary-profile.v1";

/// B2 档案：角色 = 全局主管·批前边界复核（**不是审批者**）。检查四件（§2.1）。
const GLOBAL_SUPERVISOR_BOUNDARY_PROFILE_TEXT: &str = "你是本项目的「全局主管」，职责是**批前边界复核**——在用户批准方案、动手执行之前，读一遍用户的目标和这份方案，给一句人话意见：范围和目标对不对得上。你不是审批者：你的意见不拦任何事，批不批、动不动手都在用户手里。重点看四件：\n1. **目标与方案对不对得上**：用户明说要动手改东西（改文件/做功能），方案却是纯建议、一个文件都不改（允许改动的写根为空）——这是最该点破的错配，务必直说「你要动手，这方案不会改任何文件」；\n2. **越界苗头**：方案步骤里若出现测试项目之外的路径、写 ~/.codex、git push、删除不可逆数据等字样，点名提醒；\n3. **步骤与验收齐不齐**：有没有明确步骤、改完怎么验；\n4. **风险漏报**：明显该提的风险方案里没提。\n保守：证据不足、拿不准就说拿不准（verdict 用 caution），别脑补、别夸大；全中文、人话、简短。";

/// B2 回程契约段（确定性文本）。
const GLOBAL_SUPERVISOR_BOUNDARY_CONTRACT_TEXT: &str = "回程契约（务必遵守）：最后输出**且仅输出**一个 ```json 代码块，严格形如 {\"verdict\":\"looks_ok|mismatch|caution\",\"points\":[\"一句话点评\",\"...\"],\"summary\":\"总评一两句\"}。verdict：looks_ok=目标与方案对得上、没明显问题；mismatch=目标与方案对不上（如用户要动手却纯建议、写根空）；caution=有要留意的地方或拿不准。points 放你点破的短句（没有就空数组）。不要在这个 json 块之后再写任何字。";

/// B2 LM 输出投影（serde 全 default 软着陆）。
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct BoundaryReviewJson {
    #[serde(default)]
    pub(crate) verdict: String,
    #[serde(default)]
    pub(crate) points: Vec<String>,
    #[serde(default)]
    pub(crate) summary: String,
}

/// 从主管最后消息抠出并解析边界意见 json（复用 B1 同款抠取器·软着陆）。
pub(crate) fn parse_boundary_review(raw: &str) -> Option<BoundaryReviewJson> {
    let block = crate::consultant_extract_json_block(raw)?;
    serde_json::from_str::<BoundaryReviewJson>(&block).ok()
}

/// 词表归一化（保守向）：verdict 未知/审批腔 → caution（拿不准当拿不准·不给错信号）。
pub(crate) fn normalize_boundary_verdict(raw: &str) -> String {
    match raw.trim() {
        "looks_ok" | "mismatch" | "caution" => raw.trim().to_string(),
        _ => "caution".to_string(),
    }
}

/// B2 复核输入（全从盘读·一份 pending 方案的要点）。
#[derive(Debug, Clone, Default)]
pub(crate) struct BoundaryReviewInput {
    pub(crate) project_id: String,
    pub(crate) proposal_title: String,
    pub(crate) user_goal: String,
    pub(crate) goal_summary: String,
    pub(crate) proposed_steps: Vec<String>,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) allowed_role_ids: Vec<String>,
    pub(crate) allowed_tools: Vec<String>,
    pub(crate) allowed_checks: Vec<String>,
    pub(crate) acceptance_criteria: Vec<String>,
    pub(crate) risks: Vec<(String, String)>, // (severity, summary)
}

/// 读盘组 B2 输入：按 proposal_id 从 proposal store 定位一份方案（不限状态——前端只对今天的 pending
/// 触发，后端只管按 id 取）。找不到 → Err（调用方软着陆成 unavailable·不落盘）。
pub(crate) fn load_boundary_review_input(
    state_path: &Path,
    proposal_id: &str,
) -> Result<BoundaryReviewInput, String> {
    let timestamp_ms = crate::unix_timestamp_ms();
    let store = crate::project_consultation_proposal_store::load_store(state_path, timestamp_ms)
        .map_err(|error| format!("读方案库失败：{error}"))?;
    let proposal = store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == proposal_id)
        .ok_or_else(|| "没找到这份方案（可能已被替换或清理）".to_string())?;
    Ok(BoundaryReviewInput {
        project_id: proposal.project_id.clone(),
        proposal_title: proposal.title.clone(),
        user_goal: proposal.user_goal.clone(),
        goal_summary: proposal.goal_summary.clone(),
        proposed_steps: proposal.proposed_steps.clone(),
        allowed_write_roots: proposal.scope_draft.allowed_write_roots.clone(),
        allowed_role_ids: proposal.scope_draft.allowed_role_ids.clone(),
        allowed_tools: proposal.scope_draft.allowed_tools.clone(),
        allowed_checks: proposal.scope_draft.allowed_checks.clone(),
        acceptance_criteria: proposal.acceptance_criteria.clone(),
        risks: proposal
            .risks
            .iter()
            .map(|risk| (risk.severity.clone(), risk.summary.clone()))
            .collect(),
    })
}

/// 组 B2 prompt（档案 + 盘上方案要点 + 契约段·确定性拼接不经 LM）。
pub(crate) fn build_boundary_prompt(input: &BoundaryReviewInput) -> String {
    let mut sections: Vec<String> = vec![GLOBAL_SUPERVISOR_BOUNDARY_PROFILE_TEXT.to_string()];
    // 用户目标（原话·判「要不要动手」的最关键素材）。
    let mut goal_block = format!("【用户目标（原话）】{}", clip(&input.user_goal, 600));
    if !input.goal_summary.trim().is_empty() {
        goal_block.push_str(&format!(
            "\n方案对目标的复述：{}",
            clip(&input.goal_summary, 400)
        ));
    }
    sections.push(goal_block);
    // 方案要点。
    let mut steps_block = format!("【方案：{}】步骤：", clip(&input.proposal_title, 200));
    if input.proposed_steps.is_empty() {
        steps_block.push_str("（方案没列具体步骤）");
    } else {
        for (index, step) in input.proposed_steps.iter().take(12).enumerate() {
            steps_block.push_str(&format!("\n{}. {}", index + 1, clip(step, 300)));
        }
    }
    sections.push(steps_block);
    // 范围（写根/角色/工具/checks）——写根空是「纯建议」的硬信号。
    let mut scope_block = String::from("【方案范围】");
    scope_block.push_str(&format!(
        "\n允许改动的文件范围（写根）：{}",
        if input.allowed_write_roots.is_empty() {
            "（空——这方案不会改任何文件！用户目标若是要动手，这就是错配）".to_string()
        } else {
            input.allowed_write_roots.join("；")
        }
    ));
    if !input.allowed_role_ids.is_empty() {
        scope_block.push_str(&format!("\n角色：{}", input.allowed_role_ids.join("、")));
    }
    if !input.allowed_tools.is_empty() {
        scope_block.push_str(&format!("\n工具：{}", input.allowed_tools.join("、")));
    }
    if !input.allowed_checks.is_empty() {
        scope_block.push_str(&format!("\n验证手段：{}", input.allowed_checks.join("、")));
    }
    sections.push(scope_block);
    // 验收标准。
    if input.acceptance_criteria.is_empty() {
        sections.push("【验收标准】方案没写「改完怎么验」——若该有，请点出来。".to_string());
    } else {
        sections.push(format!(
            "【验收标准】{}",
            input
                .acceptance_criteria
                .iter()
                .map(|criteria| clip(criteria, 200))
                .collect::<Vec<_>>()
                .join("；")
        ));
    }
    // 风险。
    if input.risks.is_empty() {
        sections.push(
            "【方案自列风险】方案没列任何风险——若这活明显有该提的风险，请指出漏报。".to_string(),
        );
    } else {
        let mut risks_block = "【方案自列风险】".to_string();
        for (severity, summary) in &input.risks {
            risks_block.push_str(&format!("\n- [{}] {}", severity, clip(summary, 200)));
        }
        sections.push(risks_block);
    }
    sections.push(GLOBAL_SUPERVISOR_BOUNDARY_CONTRACT_TEXT.to_string());
    sections.join("\n\n")
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RunGlobalSupervisorBoundaryReviewRequest {
    pub(crate) project_root: String,
    /// 前端只传定位键（哪份方案），不传内容（输入全从盘读·不收转述）。
    pub(crate) proposal_id: String,
    #[serde(default)]
    pub(crate) force: bool,
}

/// 返回结构（**任何失败不 Err 断面板**）：status="ready"（意见在 review 里）| "unavailable"（reason 人话·可 [重试]）。
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GlobalSupervisorBoundaryReviewOutcome {
    pub(crate) status: String,
    pub(crate) review: Option<GlobalSupervisorBoundaryReviewRecord>,
    pub(crate) reason: Option<String>,
    pub(crate) warnings: Vec<String>,
}

/// B2 核心（同步·可注入 consult 供单测 stub 计次）。
/// 幂等（成本护栏）：同 proposal_id 已有记录（**含 unavailable**）且 !force → 直接返回、不 consult；
/// [重试] 走 force=true。
pub(crate) fn run_global_supervisor_boundary_review_core<F>(
    state_path: &Path,
    request: &RunGlobalSupervisorBoundaryReviewRequest,
    canonical_project_id: &str,
    consult: F,
) -> GlobalSupervisorBoundaryReviewOutcome
where
    F: Fn(&str, &str) -> Result<String, String>,
{
    let timestamp_ms = crate::unix_timestamp_ms();
    let mut warnings: Vec<String> = Vec::new();
    if request.proposal_id.trim().is_empty() {
        return GlobalSupervisorBoundaryReviewOutcome {
            status: "unavailable".to_string(),
            review: None,
            reason: Some("缺 proposal_id（不知道要看哪份方案）".to_string()),
            warnings,
        };
    }
    // 1. 幂等命中：已有 canonical 记录且非 force → 原样返回（不 consult·不写盘）。
    // 同键 foreign 记录不得返回、覆盖或泄露。
    let (store, load_warnings) =
        global_supervisor_review_store::load_store_soft(state_path, timestamp_ms);
    warnings.extend(load_warnings);
    if let Some(blocked) =
        reject_foreign_same_key_boundary_review(&store, &request.proposal_id, canonical_project_id)
    {
        return blocked;
    }
    if !request.force {
        if let Some(existing) =
            find_canonical_boundary_review(&store, &request.proposal_id, canonical_project_id)
        {
            return GlobalSupervisorBoundaryReviewOutcome {
                status: existing.status.clone(),
                reason: existing.unavailable_reason.clone(),
                review: Some(existing.clone()),
                warnings,
            };
        }
    }
    // 2. 读盘组输入（方案找不到 → unavailable、不落盘：键不对落了也是垃圾）。
    let input = match load_boundary_review_input(state_path, &request.proposal_id) {
        Ok(input) => input,
        Err(error) => {
            return GlobalSupervisorBoundaryReviewOutcome {
                status: "unavailable".to_string(),
                review: None,
                reason: Some(format!("边界意见不可用：{error}")),
                warnings,
            };
        }
    };
    if input.project_id != canonical_project_id {
        return GlobalSupervisorBoundaryReviewOutcome {
            status: "unavailable".to_string(),
            review: None,
            reason: Some("边界意见不可用：方案不属于当前项目".to_string()),
            warnings,
        };
    }
    let prompt = build_boundary_prompt(&input);
    // 3. consult（只读）→ 解析 → 归一化；失败落「不可用」记录（可重试）。
    let base_record = GlobalSupervisorBoundaryReviewRecord {
        review_id: format!("global-supervisor-boundary-review:{}", request.proposal_id),
        project_id: canonical_project_id.to_string(),
        proposal_id: request.proposal_id.clone(),
        model: GLOBAL_SUPERVISOR_MODEL_LABEL.to_string(),
        profile_version: GLOBAL_SUPERVISOR_BOUNDARY_PROFILE_VERSION.to_string(),
        ..Default::default()
    };
    let record = match consult(&request.project_root, &prompt) {
        Ok(raw) => match parse_boundary_review(&raw) {
            Some(parsed) => GlobalSupervisorBoundaryReviewRecord {
                status: "ready".to_string(),
                verdict: normalize_boundary_verdict(&parsed.verdict),
                points: parsed
                    .points
                    .into_iter()
                    .map(|point| point.trim().to_string())
                    .filter(|point| !point.is_empty())
                    .collect(),
                summary: parsed.summary.trim().to_string(),
                unavailable_reason: None,
                ..base_record.clone()
            },
            None => GlobalSupervisorBoundaryReviewRecord {
                status: "unavailable".to_string(),
                unavailable_reason: Some("全局主管没按契约交回意见 json（可重试）".to_string()),
                ..base_record.clone()
            },
        },
        Err(error) => GlobalSupervisorBoundaryReviewRecord {
            status: "unavailable".to_string(),
            unavailable_reason: Some(crate::run_error_translation::humanize_error_for_display(
                &error,
            )),
            ..base_record.clone()
        },
    };
    // 4. 落库（唯一写入面·只碰 boundary_reviews）。写失败也不 Err——意见还在返回体里。
    match global_supervisor_review_store::upsert_boundary_review(
        state_path,
        record.clone(),
        "global_supervisor_agent",
        timestamp_ms,
    ) {
        Ok(_) => {}
        Err(error) => warnings.push(format!("边界意见落库失败（意见仍返回）：{error}")),
    }
    GlobalSupervisorBoundaryReviewOutcome {
        status: record.status.clone(),
        reason: record.unavailable_reason.clone(),
        review: Some(record),
        warnings,
    }
}

#[tauri::command]
pub(crate) async fn run_global_supervisor_boundary_review(
    request: RunGlobalSupervisorBoundaryReviewRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<GlobalSupervisorBoundaryReviewOutcome, String> {
    let canonical_project_id = resolve_global_supervisor_canonical_project_id(
        state.m1_project_index_read_port(),
        &request.project_root,
    )?;
    // 真 consult 长耗时 → spawn_blocking 不冻 UI（同 B1/咨询范本）；path 在 await 前取（不用死锚默认）。
    let path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_global_supervisor_boundary_review_core(
            &path,
            &request,
            &canonical_project_id,
            |project_root, prompt| {
                crate::codex_local_runner::readonly_codex_consult(
                    project_root,
                    prompt,
                    Some(GLOBAL_SUPERVISOR_CONSULT_TIMEOUT_MS),
                )
            },
        )
    })
    .await
    .map_err(|error| format!("边界意见执行线程异常：{error}"))
}

fn resolve_global_supervisor_canonical_project_id(
    port: Option<&dyn crate::m1_project_index::M1ProjectIndexReadPort>,
    project_root: &str,
) -> Result<String, String> {
    let port =
        port.ok_or_else(|| crate::m1_project_index::M1_PROJECT_INDEX_UNAVAILABLE.to_string())?;
    port.resolve_exact_alias(project_root)
        .map(|project_id| project_id.as_str().to_string())
        .map_err(|error| error.code)
}

fn find_canonical_review<'a>(
    store: &'a global_supervisor_review_store::GlobalSupervisorReviewStoreV1,
    workflow_id: &str,
    chain_started_at: &str,
    canonical_project_id: &str,
) -> Option<&'a GlobalSupervisorReviewRecord> {
    store.reviews.iter().find(|review| {
        review.workflow_id == workflow_id
            && review.chain_started_at == chain_started_at
            && review.project_id == canonical_project_id
    })
}

fn reject_foreign_same_key_review(
    store: &global_supervisor_review_store::GlobalSupervisorReviewStoreV1,
    workflow_id: &str,
    chain_started_at: &str,
    canonical_project_id: &str,
) -> Option<GlobalSupervisorReviewOutcome> {
    let has_canonical = store.reviews.iter().any(|review| {
        review.workflow_id == workflow_id
            && review.chain_started_at == chain_started_at
            && review.project_id == canonical_project_id
    });
    if has_canonical {
        return None;
    }
    let has_foreign = store.reviews.iter().any(|review| {
        review.workflow_id == workflow_id
            && review.chain_started_at == chain_started_at
            && review.project_id != canonical_project_id
    });
    if !has_foreign {
        return None;
    }
    Some(GlobalSupervisorReviewOutcome {
        status: "unavailable".to_string(),
        review: None,
        reason: Some("复核不可用：已有其他项目的同键复核记录".to_string()),
        warnings: Vec::new(),
    })
}

fn find_canonical_boundary_review<'a>(
    store: &'a global_supervisor_review_store::GlobalSupervisorReviewStoreV1,
    proposal_id: &str,
    canonical_project_id: &str,
) -> Option<&'a GlobalSupervisorBoundaryReviewRecord> {
    store.boundary_reviews.iter().find(|review| {
        review.proposal_id == proposal_id && review.project_id == canonical_project_id
    })
}

fn reject_foreign_same_key_boundary_review(
    store: &global_supervisor_review_store::GlobalSupervisorReviewStoreV1,
    proposal_id: &str,
    canonical_project_id: &str,
) -> Option<GlobalSupervisorBoundaryReviewOutcome> {
    let has_canonical = store.boundary_reviews.iter().any(|review| {
        review.proposal_id == proposal_id && review.project_id == canonical_project_id
    });
    if has_canonical {
        return None;
    }
    let has_foreign = store.boundary_reviews.iter().any(|review| {
        review.proposal_id == proposal_id && review.project_id != canonical_project_id
    });
    if !has_foreign {
        return None;
    }
    Some(GlobalSupervisorBoundaryReviewOutcome {
        status: "unavailable".to_string(),
        review: None,
        reason: Some("边界意见不可用：已有其他项目的同键边界意见".to_string()),
        warnings: Vec::new(),
    })
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

    /// 最小 fixture store：一轮链（started 1000/ended 2000）+ 2 份按 dispatch_id 匹配的口供
    /// + 上轮口供 + 2 个本轮任务节点。自包含手写、不依赖 lib.rs helper（worker_report 先例）。
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
                    "workflow_id": "wf-1", "work_item_id": "wi-1", "dispatch_id": "d-current-1", "created_at": "1500",
                    "executed_what": "建了 index.html 小游戏", "changed_what": "/t/index.html",
                    "acceptance_status": "reported_completed",
                    "evidence_refs": ["文件存在且能打开"], "open_issues": []
                },
                {
                    "event_id": "r2", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-2", "dispatch_id": "d-current-2", "created_at": "1800",
                    "executed_what": "无法启动浏览器，未完成手动验收", "changed_what": "（无产出）",
                    "acceptance_status": "needs_rework",
                    "evidence_refs": [], "open_issues": ["浏览器起不来"]
                },
                {
                    "event_id": "r-out", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-old", "dispatch_id": "d-old", "created_at": "500",
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
                "nodes": [
                    {"node_id": "wf-1:node:codex-dev", "state": "completed", "dispatch_id": null, "message": null},
                    {"node_id": "wf-1:node:task:aaa", "state": "completed", "dispatch_id": "d-current-1", "message": null},
                    {"node_id": "wf-1:node:task:bbb", "state": "completed", "dispatch_id": "d-current-2", "message": null}
                ]
            }]
        });
        let path = dir.join("workflow-state.v0.json");
        fs::write(&path, serde_json::to_string_pretty(&store).unwrap()).expect("write state");
        path
    }

    /// 案发形状：停链后 1ms 才落的本轮口供，和全局 nodes 中旧轮遗留的 completed。
    fn write_round_scope_regression_fixture_state(dir: &Path, project_root: &str) -> PathBuf {
        let pid = crate::project_id(project_root);
        let store = serde_json::json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": "seed",
            "projects": [],
            "agent_adapters": [],
            "workflows": [{"workflow_id": "wf-1"}],
            "nodes": [
                {"workflow_id": "wf-1", "node_id": "wf-1:node:task:done", "state": "completed"},
                {"workflow_id": "wf-1", "node_id": "wf-1:node:task:not-run", "state": "completed"}
            ],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "audit_events": [
                {
                    "event_id": "current-late", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "node_id": "wf-1:node:task:done",
                    "work_item_id": "wi-current", "dispatch_id": "d-current", "created_at": "2001",
                    "executed_what": "本轮任务已交口供", "changed_what": "/t/current.txt",
                    "acceptance_status": "reported_completed", "evidence_refs": [], "open_issues": []
                },
                {
                    "event_id": "old-round", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "node_id": "wf-1:node:task:done",
                    "work_item_id": "wi-old", "dispatch_id": "d-old", "created_at": "500",
                    "executed_what": "上一轮口供", "changed_what": "/t/old.txt",
                    "acceptance_status": "reported_completed", "evidence_refs": [], "open_issues": []
                }
            ],
            "capabilities": [],
            "harness_resources": [],
            "workflow_chain_runs": [{
                "chain_run_id": "chain-1", "project_id": pid, "workflow_id": "wf-1",
                "state": "stopped", "stop_requested": false,
                "started_at": "1000", "ended_at": "2000",
                "nodes": [
                    {"node_id": "wf-1:node:task:done", "state": "completed", "dispatch_id": "d-current", "message": null},
                    {"node_id": "wf-1:node:task:not-run", "state": "pending", "dispatch_id": null, "message": null}
                ]
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

    // 读盘：链轮精确匹配 + 口供按本轮 dispatch_id 圈对 + 任务节点按本轮投影。
    #[test]
    fn load_input_scopes_reports_to_this_round() {
        let dir = tmp_dir("scope");
        let path = write_fixture_state(&dir, "/p/root");
        let input = load_review_input(&path, &crate::project_id("/p/root"), "wf-1", "1000")
            .expect("输入应组出来");
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
        let err = load_review_input(&path, &crate::project_id("/p/root"), "wf-1", "9999")
            .expect_err("错轮应报");
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

    #[test]
    fn load_input_keeps_late_current_report_and_uses_this_round_task_states() {
        let dir = tmp_dir("round-scope-regression");
        let path = write_round_scope_regression_fixture_state(&dir, "/p/root");

        let input = load_review_input(&path, &crate::project_id("/p/root"), "wf-1", "1000")
            .expect("输入应组出来");

        assert_eq!(input.reports.len(), 1, "同轮 dispatch 的晚 1ms 口供应保留");
        assert_eq!(input.reports[0].work_item_id, "wi-current");
        assert!(
            input
                .reports
                .iter()
                .all(|report| report.work_item_id != "wi-old"),
            "旧轮口供不得混入"
        );
        assert_eq!(
            input.task_nodes,
            vec![
                ("wf-1:node:task:done".to_string(), "completed".to_string()),
                ("wf-1:node:task:not-run".to_string(), "pending".to_string()),
            ],
            "任务态只投影本轮 run.nodes，不能吃全局旧轮 completed"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_input_uses_narrow_time_fallback_only_when_run_has_no_dispatch_ids() {
        let dir = tmp_dir("round-scope-fallback");
        let path = write_round_scope_regression_fixture_state(&dir, "/p/root");
        let mut state: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read state"))
                .expect("parse state");
        for node in state["workflow_chain_runs"][0]["nodes"]
            .as_array_mut()
            .expect("run nodes")
        {
            node["dispatch_id"] = serde_json::Value::Null;
        }
        for event in state["audit_events"].as_array_mut().expect("audit events") {
            event["dispatch_id"] = serde_json::Value::Null;
        }
        state["audit_events"]
            .as_array_mut()
            .expect("audit events")
            .push(serde_json::json!({
                "event_id": "too-late", "event_type": "worker_structured_report_recorded",
                "workflow_id": "wf-1", "work_item_id": "wi-too-late", "created_at": "62001",
                "executed_what": "超过容差", "changed_what": "/t/late.txt",
                "acceptance_status": "reported_completed", "evidence_refs": [], "open_issues": []
            }));
        fs::write(
            &path,
            serde_json::to_string_pretty(&state).expect("serialize state"),
        )
        .expect("write state");

        let input = load_review_input(&path, &crate::project_id("/p/root"), "wf-1", "1000")
            .expect("输入应组出来");

        assert_eq!(
            input.reports.len(),
            1,
            "回退只纳入 ended_at 后 60 秒内的当前口供"
        );
        assert_eq!(input.reports[0].work_item_id, "wi-current");

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
        let first = run_global_supervisor_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            consult,
        );
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
        let second = run_global_supervisor_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            consult,
        );
        assert_eq!(second.status, "ready");
        assert_eq!(calls.get(), 1, "幂等命中不得重烧 consult");
        // force：重跑。
        let forced = RunGlobalSupervisorReviewRequest {
            force: true,
            ..request.clone()
        };
        let third = run_global_supervisor_review_core(
            &path,
            &forced,
            &crate::project_id(&forced.project_root),
            consult,
        );
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
        let outcome = run_global_supervisor_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            failing,
        );
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
        let again = run_global_supervisor_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            failing,
        );
        assert_eq!(again.status, "unavailable");
        assert_eq!(calls.get(), 1, "unavailable 也参与幂等·自动路不重烧");
        // force + 供给恢复 → ready 覆盖同键记录。
        let recovered = |_root: &str, _prompt: &str| Ok(GOOD_REVIEW.to_string());
        let forced = RunGlobalSupervisorReviewRequest {
            force: true,
            ..request.clone()
        };
        let retried = run_global_supervisor_review_core(
            &path,
            &forced,
            &crate::project_id(&forced.project_root),
            recovered,
        );
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
        let outcome = run_global_supervisor_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            no_contract,
        );
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
        let outcome2 = run_global_supervisor_review_core(
            &path,
            &missing,
            &crate::project_id(&missing.project_root),
            no_contract,
        );
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
        let outcome = run_global_supervisor_review_core(
            &state_path,
            &request,
            &crate::project_id(&request.project_root),
            |root, prompt| {
                crate::codex_local_runner::readonly_codex_consult(root, prompt, Some(420_000))
            },
        );
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

    // ========================================================================
    // B2·批前边界意见 单测（§4）
    // ========================================================================

    /// 手写一份 pending 方案 sidecar（写到 proposal store 真实 sidecar 路径·全字段·避免 serde 缺字段）。
    /// write_roots 空 = 纯建议方案（money-shot：目标要动手 vs 写根空 = mismatch）。
    fn write_proposal_fixture(
        state_path: &Path,
        project_root: &str,
        proposal_id: &str,
        user_goal: &str,
        write_roots: &[&str],
    ) {
        let sidecar = crate::project_consultation_proposal_store::sidecar_path(state_path)
            .expect("proposal sidecar path");
        if let Some(parent) = sidecar.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let store = serde_json::json!({
            "schema_version": "project_consultation_proposal_store.v1",
            "revision": 1,
            "proposals": [{
                "proposal_id": proposal_id,
                "schema_version": "project_consultation_proposal.v1",
                "project_id": crate::project_id(project_root),
                "workflow_id": "wf-1",
                "title": "加个暂停功能",
                "user_goal": user_goal,
                "goal_summary": "给游戏加暂停键",
                "proposed_steps": ["建议你考虑加暂停", "可以研究一下键盘事件监听"],
                "scope_draft": {
                    "allowed_role_ids": [],
                    "allowed_agent_ids": [],
                    "allowed_read_roots": [],
                    "allowed_write_roots": write_roots,
                    "allowed_tools": [],
                    "allowed_checks": [],
                    "allowed_task_package_kinds": [],
                    "stop_conditions": [],
                    "max_worker_dispatches": null,
                    "max_runtime_minutes": null
                },
                "risks": [],
                "acceptance_criteria": ["打开游戏能按 P 键暂停"],
                "status": "pending_user_confirmation",
                "plan_authorization_id": null,
                "created_by_role": "project_consultant",
                "suggest_workflow": false,
                "created_at_ms": 1000,
                "updated_at_ms": 1000
            }],
            "decisions": [],
            "audit_events": [],
            "updated_at_ms": 1000,
            "warnings": []
        });
        fs::write(&sidecar, serde_json::to_string_pretty(&store).unwrap())
            .expect("write proposal sidecar");
    }

    const GOOD_BOUNDARY: &str = "看了下方案。\n```json\n{\"verdict\":\"mismatch\",\"points\":[\"你说要动手加暂停功能，但这方案的允许写根是空的——它不会改任何文件\",\"步骤是「建议你考虑」这类话，不是能落地执行的动作\"],\"summary\":\"目标要动手、方案是纯建议，对不上；别急着批\"}\n```";

    // §4·schema 三态①：合法块解析 + 归一化透传。
    #[test]
    fn boundary_parses_valid_block() {
        let parsed = parse_boundary_review(GOOD_BOUNDARY).expect("合法块应解析");
        assert_eq!(parsed.verdict, "mismatch");
        assert_eq!(parsed.points.len(), 2);
        assert!(parsed.summary.contains("对不上"));
    }

    // §4·schema 三态②：缺字段 default 容忍 + 归一化保守向（未知/审批腔 → caution）。
    #[test]
    fn boundary_missing_fields_default_and_normalize_conservative() {
        let parsed = parse_boundary_review("```json\n{\"summary\":\"只给了总评\"}\n```")
            .expect("缺字段也解析");
        assert_eq!(parsed.summary, "只给了总评");
        assert!(parsed.points.is_empty());
        assert_eq!(normalize_boundary_verdict(""), "caution");
        assert_eq!(
            normalize_boundary_verdict("approved"),
            "caution",
            "审批腔归 caution"
        );
        assert_eq!(normalize_boundary_verdict("looks_ok"), "looks_ok");
        assert_eq!(normalize_boundary_verdict("mismatch"), "mismatch");
        assert_eq!(normalize_boundary_verdict("caution"), "caution");
    }

    // §4·schema 三态③：坏 json / 无块 → None（软着陆）。
    #[test]
    fn boundary_broken_or_missing_block_is_none() {
        assert!(parse_boundary_review("没有块").is_none());
        assert!(parse_boundary_review("```json\n{坏的\n```").is_none());
    }

    // 读盘 + prompt 组装：用户目标进 prompt、写根空触发硬信号提示、契约段在。
    #[test]
    fn boundary_load_input_and_prompt_grounded() {
        let dir = tmp_dir("b2input");
        let path = write_fixture_state(&dir, "/p/root"); // 复用 B1 的 workflow-state 骨架
        write_proposal_fixture(
            &path,
            "/p/root",
            "prop-1",
            "帮我把游戏加个暂停功能，按 P 键暂停",
            &[],
        );
        let input = load_boundary_review_input(&path, "prop-1").expect("输入应组出来");
        assert!(input.user_goal.contains("暂停"));
        assert!(input.allowed_write_roots.is_empty(), "纯建议方案写根空");
        let prompt = build_boundary_prompt(&input);
        assert!(prompt.contains("按 P 键暂停"), "用户目标原话进 prompt");
        assert!(
            prompt.contains("不会改任何文件"),
            "写根空 → 硬信号提示进 prompt"
        );
        assert!(prompt.contains("批前边界复核"), "档案在");
        assert!(prompt.contains("回程契约"), "契约段在");
        // 方案不存在 → Err 人话。
        let err = load_boundary_review_input(&path, "prop-x").expect_err("错 id 应报");
        assert!(err.contains("没找到这份方案"), "人话：{err}");
        // 空验收/空风险分支（纯函数·手搭 input·不落盘）：prompt 点出「没写验收」「没列风险」。
        let bare = BoundaryReviewInput {
            user_goal: "随便看看".to_string(),
            goal_summary: "看看".to_string(),
            proposed_steps: vec!["看一眼".to_string()],
            ..Default::default()
        };
        let bare_prompt = build_boundary_prompt(&bare);
        assert!(bare_prompt.contains("没写「改完怎么验」"), "空验收提示");
        assert!(bare_prompt.contains("没列任何风险"), "空风险提示");
        let _ = fs::remove_dir_all(dir);
    }

    // §4·money-shot：纯建议方案 → mismatch 意见 + 落库 + 幂等命中不重跑（stub 计次=1）+ force 重跑（=2）。
    #[test]
    fn boundary_core_ready_mismatch_idempotent_and_force() {
        let dir = tmp_dir("b2core");
        let path = write_fixture_state(&dir, "/p/root");
        write_proposal_fixture(&path, "/p/root", "prop-1", "帮我动手加暂停功能", &[]);
        let calls = Cell::new(0usize);
        let consult = |_root: &str, _prompt: &str| {
            calls.set(calls.get() + 1);
            Ok(GOOD_BOUNDARY.to_string())
        };
        let request = RunGlobalSupervisorBoundaryReviewRequest {
            project_root: "/p/root".to_string(),
            proposal_id: "prop-1".to_string(),
            force: false,
        };
        let first = run_global_supervisor_boundary_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            consult,
        );
        assert_eq!(first.status, "ready", "{:?}", first.reason);
        assert_eq!(calls.get(), 1);
        let review = first.review.expect("ready 应带记录");
        assert_eq!(review.verdict, "mismatch", "点破目标 vs 纯建议错配");
        assert_eq!(review.points.len(), 2);
        assert_eq!(review.model, GLOBAL_SUPERVISOR_MODEL_LABEL);
        assert_eq!(
            review.profile_version, GLOBAL_SUPERVISOR_BOUNDARY_PROFILE_VERSION,
            "B2 档案版本落记录"
        );
        // 幂等命中：第二次非 force 不再 consult。
        let second = run_global_supervisor_boundary_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            consult,
        );
        assert_eq!(second.status, "ready");
        assert_eq!(calls.get(), 1, "幂等命中不得重烧 consult");
        // force 重跑。
        let forced = RunGlobalSupervisorBoundaryReviewRequest {
            force: true,
            ..request.clone()
        };
        let third = run_global_supervisor_boundary_review_core(
            &path,
            &forced,
            &crate::project_id(&forced.project_root),
            consult,
        );
        assert_eq!(third.status, "ready");
        assert_eq!(calls.get(), 2, "force 才重跑");
        // 落库单条（同 proposal_id 覆盖不追加）+ B2 内嵌审计在。
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 0);
        assert_eq!(store.boundary_reviews.len(), 1, "同 proposal_id 覆盖不追加");
        assert!(store
            .boundary_audit_events
            .iter()
            .any(|event| event.event_type == "global_supervisor_boundary_review_recorded"));
        let _ = fs::remove_dir_all(dir);
    }

    // §4：供给类失败 → 人话（剥前缀）+ 落「不可用」可重试；方案不存在 → unavailable 但不落盘。
    #[test]
    fn boundary_provider_failure_humanized_and_missing_proposal_not_recorded() {
        let dir = tmp_dir("b2fail");
        let path = write_fixture_state(&dir, "/p/root");
        write_proposal_fixture(
            &path,
            "/p/root",
            "prop-1",
            "帮我动手加暂停功能",
            &["/p/root"],
        );
        let calls = Cell::new(0usize);
        let failing = |_root: &str, _prompt: &str| {
            calls.set(calls.get() + 1);
            Err("codex_provider_unavailable:codex 额度用完了，明天再试".to_string())
        };
        let request = RunGlobalSupervisorBoundaryReviewRequest {
            project_root: "/p/root".to_string(),
            proposal_id: "prop-1".to_string(),
            force: false,
        };
        let outcome = run_global_supervisor_boundary_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            failing,
        );
        assert_eq!(outcome.status, "unavailable");
        let reason = outcome.reason.clone().expect("应带人话原因");
        assert!(reason.contains("额度用完"), "供给类人话直取：{reason}");
        assert!(
            !reason.contains("codex_provider_unavailable:"),
            "前缀应剥掉"
        );
        // unavailable 也落记录（可重试）+ 参与幂等（自动路不重烧）。
        let again = run_global_supervisor_boundary_review_core(
            &path,
            &request,
            &crate::project_id(&request.project_root),
            failing,
        );
        assert_eq!(again.status, "unavailable");
        assert_eq!(calls.get(), 1, "unavailable 也参与幂等·自动路不重烧");
        // force + 供给恢复 → ready 覆盖同 proposal_id。
        let recovered = |_root: &str, _prompt: &str| Ok(GOOD_BOUNDARY.to_string());
        let forced = RunGlobalSupervisorBoundaryReviewRequest {
            force: true,
            ..request.clone()
        };
        let retried = run_global_supervisor_boundary_review_core(
            &path,
            &forced,
            &crate::project_id(&forced.project_root),
            recovered,
        );
        assert_eq!(retried.status, "ready");
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_999);
        assert_eq!(store.boundary_reviews.len(), 1, "同键覆盖不追加");
        // 方案不存在：unavailable + 不落盘（键不对落了也是垃圾）。
        let missing = RunGlobalSupervisorBoundaryReviewRequest {
            project_root: "/p/root".to_string(),
            proposal_id: "prop-nope".to_string(),
            force: false,
        };
        let out2 = run_global_supervisor_boundary_review_core(
            &path,
            &missing,
            &crate::project_id(&missing.project_root),
            recovered,
        );
        assert_eq!(out2.status, "unavailable");
        assert!(out2.reason.unwrap_or_default().contains("没找到这份方案"));
        let (store2, _) = global_supervisor_review_store::load_store_soft(&path, 9_998);
        assert!(
            global_supervisor_review_store::find_boundary_review(&store2, "prop-nope").is_none(),
            "错 id 不落盘"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4·真跑（单独步·#[ignore]·固定测试项目·额度在）：对盘上真 pending 方案出 grounded 边界意见。
    // 显式 `cargo test --lib global_supervisor_boundary_review_real_run -- --ignored --nocapture`。
    // 最佳夹具 = 盘上纯建议方案（目标要动手 vs 写根空），意见应点破 mismatch（B2 money-shot）。
    #[test]
    #[ignore = "B2 global supervisor boundary review: real read-only opinion on a pending proposal in the test project (user present, quota available)"]
    fn global_supervisor_boundary_review_real_run() {
        let state_path = crate::default_workflow_state_path();
        assert!(
            state_path.exists(),
            "真 store 应存在：{}",
            state_path.display()
        );
        let auth_path = PathBuf::from(std::env::var("HOME").expect("HOME"))
            .join(".codex")
            .join("auth.json");
        let auth_before = fs::metadata(&auth_path).and_then(|m| m.modified()).ok();
        // 从盘上取一份方案（优先纯建议方案=写根空·money-shot；否则取最新一份）。
        let store = crate::project_consultation_proposal_store::load_store(
            &state_path,
            crate::unix_timestamp_ms(),
        )
        .expect("load proposal store");
        // 可选定向：设 B2_REAL_PROPOSAL_ID 精确挑一份方案（核实物/复现 money-shot 用）；否则默认取
        // 最新纯建议方案（写根空），再兜底最新一份。默认行为不变=向后兼容。
        let forced_id = std::env::var("B2_REAL_PROPOSAL_ID").ok();
        let proposal = match forced_id.as_deref() {
            Some(id) if !id.trim().is_empty() => store
                .proposals
                .iter()
                .find(|proposal| proposal.proposal_id == id)
                .unwrap_or_else(|| panic!("B2_REAL_PROPOSAL_ID={id} 不在盘上"))
                .clone(),
            _ => store
                .proposals
                .iter()
                .filter(|proposal| proposal.scope_draft.allowed_write_roots.is_empty())
                .max_by_key(|proposal| proposal.created_at_ms)
                .or_else(|| {
                    store
                        .proposals
                        .iter()
                        .max_by_key(|proposal| proposal.created_at_ms)
                })
                .expect("盘上应有至少一份方案（先真机说一个目标出方案）")
                .clone(),
        };
        println!(
            "[B2_REAL] 方案 id={} 标题={} 写根空={}",
            proposal.proposal_id,
            proposal.title,
            proposal.scope_draft.allowed_write_roots.is_empty()
        );
        let request = RunGlobalSupervisorBoundaryReviewRequest {
            project_root: "/Users/yoyi/codex-workflow-mario-test".to_string(),
            proposal_id: proposal.proposal_id.clone(),
            force: true,
        };
        let outcome = run_global_supervisor_boundary_review_core(
            &state_path,
            &request,
            &crate::project_id(&request.project_root),
            |root, prompt| {
                crate::codex_local_runner::readonly_codex_consult(root, prompt, Some(420_000))
            },
        );
        println!(
            "[B2_REAL] status={} reason={:?}",
            outcome.status, outcome.reason
        );
        let review = outcome.review.clone().expect("应带记录");
        println!(
            "[B2_REAL] verdict={} summary={}",
            review.verdict, review.summary
        );
        for point in &review.points {
            println!("[B2_REAL] - {point}");
        }
        assert_eq!(
            outcome.status, "ready",
            "真边界意见应出：{:?}",
            outcome.reason
        );
        assert!(!review.summary.trim().is_empty(), "总评非空");
        // 落库 + B2 内嵌审计在。
        let (saved_store, _) = global_supervisor_review_store::load_store_soft(&state_path, 0);
        assert!(
            global_supervisor_review_store::find_boundary_review(
                &saved_store,
                &proposal.proposal_id
            )
            .is_some(),
            "记录应落盘"
        );
        assert!(saved_store
            .boundary_audit_events
            .iter()
            .any(|event| event.event_type == "global_supervisor_boundary_review_recorded"));
        let auth_after = fs::metadata(&auth_path).and_then(|m| m.modified()).ok();
        assert_eq!(auth_before, auth_after, ".codex 凭据不许被碰");
    }

    fn write_fixture_state_with_project_id(dir: &Path, project_id: &str) -> PathBuf {
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
                    "workflow_id": "wf-1", "work_item_id": "wi-1", "dispatch_id": "d-current-1", "created_at": "1500",
                    "executed_what": "建了 index.html 小游戏", "changed_what": "/t/index.html",
                    "acceptance_status": "reported_completed",
                    "evidence_refs": ["文件存在且能打开"], "open_issues": []
                },
                {
                    "event_id": "r2", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-2", "dispatch_id": "d-current-2", "created_at": "1800",
                    "executed_what": "无法启动浏览器，未完成手动验收", "changed_what": "（无产出）",
                    "acceptance_status": "needs_rework",
                    "evidence_refs": [], "open_issues": ["浏览器起不来"]
                }
            ],
            "capabilities": [],
            "harness_resources": [],
            "workflow_chain_runs": [{
                "chain_run_id": "chain-1", "project_id": project_id, "workflow_id": "wf-1",
                "state": "completed", "stop_requested": false,
                "started_at": "1000", "ended_at": "2000",
                "nodes": [
                    {"node_id": "wf-1:node:codex-dev", "state": "completed", "dispatch_id": null, "message": null},
                    {"node_id": "wf-1:node:task:aaa", "state": "completed", "dispatch_id": "d-current-1", "message": null},
                    {"node_id": "wf-1:node:task:bbb", "state": "completed", "dispatch_id": "d-current-2", "message": null}
                ]
            }]
        });
        let path = dir.join("workflow-state.v0.json");
        fs::write(&path, serde_json::to_string_pretty(&store).unwrap()).expect("write state");
        path
    }

    fn write_proposal_fixture_with_project_id(
        state_path: &Path,
        project_id: &str,
        proposal_id: &str,
        user_goal: &str,
        write_roots: &[&str],
    ) {
        let sidecar = crate::project_consultation_proposal_store::sidecar_path(state_path)
            .expect("proposal sidecar path");
        if let Some(parent) = sidecar.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let store = serde_json::json!({
            "schema_version": "project_consultation_proposal_store.v1",
            "revision": 1,
            "proposals": [{
                "proposal_id": proposal_id,
                "schema_version": "project_consultation_proposal.v1",
                "project_id": project_id,
                "workflow_id": "wf-1",
                "title": "加个暂停功能",
                "user_goal": user_goal,
                "goal_summary": "给游戏加暂停键",
                "proposed_steps": ["建议你考虑加暂停", "可以研究一下键盘事件监听"],
                "scope_draft": {
                    "allowed_role_ids": [],
                    "allowed_agent_ids": [],
                    "allowed_read_roots": [],
                    "allowed_write_roots": write_roots,
                    "allowed_tools": [],
                    "allowed_checks": [],
                    "allowed_task_package_kinds": [],
                    "stop_conditions": [],
                    "max_worker_dispatches": null,
                    "max_runtime_minutes": null
                },
                "risks": [],
                "acceptance_criteria": ["打开游戏能按 P 键暂停"],
                "status": "pending_user_confirmation",
                "plan_authorization_id": null,
                "created_by_role": "project_consultant",
                "suggest_workflow": false,
                "created_at_ms": 1000,
                "updated_at_ms": 1000
            }],
            "decisions": [],
            "audit_events": [],
            "updated_at_ms": 1000,
            "warnings": []
        });
        fs::write(&sidecar, serde_json::to_string_pretty(&store).unwrap())
            .expect("write proposal sidecar");
    }

    fn write_review_sidecar(state_path: &Path, body: serde_json::Value) -> PathBuf {
        let sidecar =
            global_supervisor_review_store::sidecar_path(state_path).expect("review sidecar");
        if let Some(parent) = sidecar.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&sidecar, serde_json::to_string_pretty(&body).unwrap()).expect("write review");
        sidecar
    }

    fn write_mixed_owner_chain_state(
        dir: &Path,
        canonical: &str,
        path_derived: &str,
        foreign: &str,
    ) -> PathBuf {
        let store = serde_json::json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": "seed",
            "projects": [],
            "agent_adapters": [],
            "workflows": [{"workflow_id": "wf-1"}],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "audit_events": [
                {
                    "event_id": "path", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-path", "dispatch_id": "d-path", "created_at": "1500",
                    "executed_what": "PATH_DERIVED_SIGNAL", "changed_what": "/t/path.txt",
                    "acceptance_status": "reported_completed", "evidence_refs": [], "open_issues": []
                },
                {
                    "event_id": "foreign", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-foreign", "dispatch_id": "d-foreign", "created_at": "1500",
                    "executed_what": "FOREIGN_SIGNAL", "changed_what": "/t/foreign.txt",
                    "acceptance_status": "reported_completed", "evidence_refs": [], "open_issues": []
                },
                {
                    "event_id": "canonical", "event_type": "worker_structured_report_recorded",
                    "workflow_id": "wf-1", "work_item_id": "wi-canonical", "dispatch_id": "d-canonical", "created_at": "1500",
                    "executed_what": "CANONICAL_ONLY_SIGNAL", "changed_what": "/t/canonical.txt",
                    "acceptance_status": "reported_completed", "evidence_refs": [], "open_issues": []
                }
            ],
            "capabilities": [],
            "harness_resources": [],
            "workflow_chain_runs": [
                {
                    "chain_run_id": "chain-path", "project_id": path_derived, "workflow_id": "wf-1",
                    "state": "failed", "stop_requested": false,
                    "started_at": "1000", "ended_at": "2000",
                    "nodes": [
                        {"node_id": "wf-1:node:task:path", "state": "failed", "dispatch_id": "d-path", "message": null}
                    ]
                },
                {
                    "chain_run_id": "chain-foreign", "project_id": foreign, "workflow_id": "wf-1",
                    "state": "stopped", "stop_requested": false,
                    "started_at": "1000", "ended_at": "2000",
                    "nodes": [
                        {"node_id": "wf-1:node:task:foreign", "state": "stopped", "dispatch_id": "d-foreign", "message": null}
                    ]
                },
                {
                    "chain_run_id": "chain-canonical", "project_id": canonical, "workflow_id": "wf-1",
                    "state": "completed", "stop_requested": false,
                    "started_at": "1000", "ended_at": "2000",
                    "nodes": [
                        {"node_id": "wf-1:node:task:canonical", "state": "completed", "dispatch_id": "d-canonical", "message": null}
                    ]
                }
            ]
        });
        let path = dir.join("workflow-state.v0.json");
        fs::write(&path, serde_json::to_string_pretty(&store).unwrap()).expect("write mixed");
        path
    }

    struct ResolveExactAliasStub {
        missing_code: &'static str,
    }

    impl crate::m1_project_index::M1ProjectIndexReadPort for ResolveExactAliasStub {
        fn resolve_canonical_project_id(
            &self,
            _claim: &str,
        ) -> Result<
            crate::m1_project_index::M1ProjectId,
            crate::m1_project_index::M1ProjectIndexError,
        > {
            Err(crate::m1_project_index::M1ProjectIndexError::new(
                self.missing_code,
            ))
        }

        fn resolve_exact_alias(
            &self,
            alias: &str,
        ) -> Result<
            crate::m1_project_index::M1ProjectId,
            crate::m1_project_index::M1ProjectIndexError,
        > {
            if alias.trim().is_empty() {
                return Err(crate::m1_project_index::M1ProjectIndexError::new(
                    "m1_alias_malformed",
                ));
            }
            Err(crate::m1_project_index::M1ProjectIndexError::new(
                self.missing_code,
            ))
        }

        fn resolve_project_root_ref(
            &self,
            _project_root_ref: &crate::m1_project_index::M1ProjectRootRef,
        ) -> Result<
            crate::m1_project_index::M1ProjectId,
            crate::m1_project_index::M1ProjectIndexError,
        > {
            Err(crate::m1_project_index::M1ProjectIndexError::new(
                self.missing_code,
            ))
        }
    }

    #[test]
    fn global_supervisor_canonical_chain_run_preferred_over_path_derived_and_foreign() {
        let dir = tmp_dir("mixed-chain");
        let root = "/tmp/m6p00-gs-mixed-chain";
        let canonical = "project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaa40";
        let path_derived = crate::project_id(root);
        let foreign = "project:ffffffff-ffff-4fff-8fff-ffffffffffff";
        assert_ne!(canonical, path_derived.as_str());
        let path = write_mixed_owner_chain_state(&dir, canonical, &path_derived, foreign);
        let input = load_review_input(&path, canonical, "wf-1", "1000").expect("canonical run");
        assert_eq!(input.chain_state, "completed");
        assert_eq!(input.reports.len(), 1);
        assert_eq!(input.reports[0].executed_what, "CANONICAL_ONLY_SIGNAL");
        assert!(input
            .reports
            .iter()
            .all(|report| report.executed_what != "PATH_DERIVED_SIGNAL"
                && report.executed_what != "FOREIGN_SIGNAL"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn m6p00_project_summary_path_derived_scope_cannot_read_canonical_summary() {
        use crate::m5_orchestration_store::M5OrchestrationStore;
        use crate::m5_project_summary::{
            rebuild_project_summary, PersistentProjectSummaryPort, ProjectSummaryQueryPort,
            QueryError, SummaryConsumer,
        };

        let root = "/tmp/m6p00-project-summary-canonical";
        let canonical = "project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaa46";
        let path_derived = crate::project_id(root);
        assert_ne!(canonical, path_derived.as_str());

        let store = M5OrchestrationStore::open_in_memory().expect("in-memory M5 store");
        rebuild_project_summary(&store, canonical, 2_000).expect("canonical summary rebuild");
        let snapshot = || {
            let connection = store.connection();
            let count = connection
                .query_row("SELECT COUNT(*) FROM m5_project_summaries", [], |row| {
                    row.get::<_, i64>(0)
                })
                .expect("summary row count");
            let fields = connection
                .query_row(
                    "SELECT project_id, orchestration_id, schema_version, version, watermark_ms, \
                            summary_hash, source_refs_json, fact_count, unverified_claim_count, \
                            open_run_count, rebuilt_at_ms \
                     FROM m5_project_summaries WHERE project_id=?1",
                    [canonical],
                    |row| {
                        Ok(vec![
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, i64>(3)?.to_string(),
                            row.get::<_, i64>(4)?.to_string(),
                            row.get::<_, String>(5)?,
                            row.get::<_, String>(6)?,
                            row.get::<_, i64>(7)?.to_string(),
                            row.get::<_, i64>(8)?.to_string(),
                            row.get::<_, i64>(9)?.to_string(),
                            row.get::<_, i64>(10)?.to_string(),
                        ])
                    },
                )
                .expect("canonical summary row");
            (count, fields)
        };
        let before = snapshot();
        let port = PersistentProjectSummaryPort::new(&store);
        let path_scoped_consumer = SummaryConsumer {
            role_session_id: "role-session:m6p00:path-derived".to_string(),
            role: "global_supervisor".to_string(),
            scope_project_id: path_derived,
            expires_at_ms: 9_000,
        };

        let denied = port
            .get_summary(canonical, &path_scoped_consumer, 3_000)
            .expect_err("path-derived scope must not read canonical summary");
        assert_eq!(
            denied,
            QueryError::InsufficientPermission("cross_project_summary_denied".to_string())
        );
        assert_eq!(snapshot(), before, "denied query must be exact zero-write");

        let canonical_consumer = SummaryConsumer {
            role_session_id: "role-session:m6p00:canonical".to_string(),
            role: "global_supervisor".to_string(),
            scope_project_id: canonical.to_string(),
            expires_at_ms: 9_000,
        };
        let summary = port
            .get_summary(canonical, &canonical_consumer, 3_000)
            .expect("canonical scope may read canonical summary");
        assert_eq!(summary.project_id, canonical);
        assert_eq!(snapshot(), before, "successful query must remain read-only");
    }

    #[test]
    fn global_supervisor_boundary_foreign_proposal_zero_consult_zero_write() {
        let dir = tmp_dir("b2-foreign-proposal");
        let path = write_fixture_state(&dir, "/p/root");
        let canonical = "project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaa41";
        let foreign = "project:dddddddd-dddd-4ddd-8ddd-dddddddddddd";
        write_proposal_fixture_with_project_id(&path, foreign, "prop-1", "帮我动手加暂停功能", &[]);
        let sidecar = global_supervisor_review_store::sidecar_path(&path).expect("review sidecar");
        let before = if sidecar.exists() {
            fs::read(&sidecar).expect("before")
        } else {
            Vec::new()
        };
        let calls = Cell::new(0usize);
        let consult = |_root: &str, _prompt: &str| {
            calls.set(calls.get() + 1);
            Ok(GOOD_BOUNDARY.to_string())
        };
        let request = RunGlobalSupervisorBoundaryReviewRequest {
            project_root: "/p/root".to_string(),
            proposal_id: "prop-1".to_string(),
            force: false,
        };
        let outcome =
            run_global_supervisor_boundary_review_core(&path, &request, canonical, consult);
        assert_eq!(outcome.status, "unavailable");
        assert!(
            outcome
                .reason
                .clone()
                .unwrap_or_default()
                .contains("方案不属于当前项目"),
            "{:?}",
            outcome.reason
        );
        assert!(outcome.review.is_none(), "foreign proposal must not leak");
        assert_eq!(calls.get(), 0, "zero consult");
        let after = if sidecar.exists() {
            fs::read(&sidecar).expect("after")
        } else {
            Vec::new()
        };
        assert_eq!(after, before, "review store exact zero write");
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_000);
        assert!(store.boundary_reviews.is_empty());
        assert!(store.boundary_audit_events.is_empty());
        assert_eq!(store.revision, 0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_supervisor_foreign_review_same_key_is_not_idempotent_hit() {
        let dir = tmp_dir("b1-foreign-review");
        let canonical = "project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaa42";
        let foreign = "project:cccccccc-cccc-4ccc-8ccc-cccccccccccc";
        let path = write_fixture_state_with_project_id(&dir, canonical);
        let sidecar = write_review_sidecar(
            &path,
            serde_json::json!({
                "schema_version": "global_supervisor_review_store.v1",
                "revision": 4,
                "updated_at_ms": 1000,
                "reviews": [{
                    "review_id": "global-supervisor-review:wf-1:1000",
                    "project_id": foreign,
                    "workflow_id": "wf-1",
                    "chain_started_at": "1000",
                    "status": "ready",
                    "overall": "pass",
                    "summary": "FOREIGN_REVIEW_LEAK",
                    "suggested_action": "none",
                    "human_note": "",
                    "tasks": [],
                    "unavailable_reason": null,
                    "model": GLOBAL_SUPERVISOR_MODEL_LABEL,
                    "profile_version": GLOBAL_SUPERVISOR_PROFILE_VERSION,
                    "created_at_ms": 1000,
                    "updated_at_ms": 1000
                }],
                "audit_events": [{
                    "event_id": "seed",
                    "event_type": "global_supervisor_review_recorded",
                    "workflow_id": "wf-1",
                    "chain_started_at": "1000",
                    "review_status": "ready",
                    "actor_ref": "seed",
                    "created_at_ms": 1000
                }],
                "boundary_reviews": [],
                "boundary_audit_events": []
            }),
        );
        let before = fs::read(&sidecar).expect("before");
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
        let outcome = run_global_supervisor_review_core(&path, &request, canonical, consult);
        assert_eq!(outcome.status, "unavailable");
        assert!(outcome.review.is_none(), "must not leak foreign review");
        assert_eq!(calls.get(), 0);
        assert_eq!(fs::read(&sidecar).expect("after"), before);
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_000);
        assert_eq!(store.revision, 4);
        assert_eq!(store.reviews.len(), 1);
        assert_eq!(store.reviews[0].project_id, foreign);
        assert_eq!(store.reviews[0].summary, "FOREIGN_REVIEW_LEAK");
        assert_eq!(store.audit_events.len(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_supervisor_boundary_foreign_review_same_key_is_not_idempotent_hit() {
        let dir = tmp_dir("b2-foreign-review");
        let canonical = "project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaa43";
        let foreign = "project:bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
        let path = write_fixture_state_with_project_id(&dir, canonical);
        write_proposal_fixture_with_project_id(
            &path,
            canonical,
            "prop-1",
            "帮我动手加暂停功能",
            &[],
        );
        let sidecar = write_review_sidecar(
            &path,
            serde_json::json!({
                "schema_version": "global_supervisor_review_store.v1",
                "revision": 2,
                "updated_at_ms": 1000,
                "reviews": [],
                "audit_events": [],
                "boundary_reviews": [{
                    "review_id": "global-supervisor-boundary-review:prop-1",
                    "project_id": foreign,
                    "proposal_id": "prop-1",
                    "status": "ready",
                    "verdict": "looks_ok",
                    "points": ["FOREIGN_BOUNDARY_LEAK"],
                    "summary": "FOREIGN_BOUNDARY_LEAK",
                    "unavailable_reason": null,
                    "model": GLOBAL_SUPERVISOR_MODEL_LABEL,
                    "profile_version": GLOBAL_SUPERVISOR_BOUNDARY_PROFILE_VERSION,
                    "created_at_ms": 1000,
                    "updated_at_ms": 1000
                }],
                "boundary_audit_events": [{
                    "event_id": "seed-b2",
                    "event_type": "global_supervisor_boundary_review_recorded",
                    "proposal_id": "prop-1",
                    "review_status": "ready",
                    "actor_ref": "seed",
                    "created_at_ms": 1000
                }]
            }),
        );
        let before = fs::read(&sidecar).expect("before");
        let calls = Cell::new(0usize);
        let consult = |_root: &str, _prompt: &str| {
            calls.set(calls.get() + 1);
            Ok(GOOD_BOUNDARY.to_string())
        };
        let request = RunGlobalSupervisorBoundaryReviewRequest {
            project_root: "/p/root".to_string(),
            proposal_id: "prop-1".to_string(),
            force: false,
        };
        let outcome =
            run_global_supervisor_boundary_review_core(&path, &request, canonical, consult);
        assert_eq!(outcome.status, "unavailable");
        assert!(outcome.review.is_none());
        assert_eq!(calls.get(), 0);
        assert_eq!(fs::read(&sidecar).expect("after"), before);
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_000);
        assert_eq!(store.revision, 2);
        assert_eq!(store.boundary_reviews.len(), 1);
        assert_eq!(store.boundary_reviews[0].project_id, foreign);
        assert_eq!(store.boundary_audit_events.len(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_supervisor_canonical_b1_persists_canonical_project_id() {
        let dir = tmp_dir("b1-canonical-persist");
        let root = "/tmp/m6p00-gs-b1-canonical";
        let canonical = "project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaa44";
        assert_ne!(canonical, crate::project_id(root).as_str());
        let path = write_fixture_state_with_project_id(&dir, canonical);
        let request = RunGlobalSupervisorReviewRequest {
            project_root: root.to_string(),
            workflow_id: "wf-1".to_string(),
            chain_started_at: "1000".to_string(),
            force: false,
        };
        let outcome =
            run_global_supervisor_review_core(&path, &request, canonical, |_root, _prompt| {
                Ok(GOOD_REVIEW.to_string())
            });
        assert_eq!(outcome.status, "ready", "{:?}", outcome.reason);
        let review = outcome.review.expect("ready");
        assert_eq!(review.project_id, canonical);
        assert_ne!(review.project_id, crate::project_id(root));
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_000);
        assert_eq!(store.reviews.len(), 1);
        assert_eq!(store.reviews[0].project_id, canonical);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_supervisor_canonical_b2_persists_canonical_project_id() {
        let dir = tmp_dir("b2-canonical-persist");
        let root = "/tmp/m6p00-gs-b2-canonical";
        let canonical = "project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaa45";
        assert_ne!(canonical, crate::project_id(root).as_str());
        let path = write_fixture_state_with_project_id(&dir, canonical);
        write_proposal_fixture_with_project_id(
            &path,
            canonical,
            "prop-1",
            "帮我动手加暂停功能",
            &[],
        );
        let request = RunGlobalSupervisorBoundaryReviewRequest {
            project_root: root.to_string(),
            proposal_id: "prop-1".to_string(),
            force: false,
        };
        let outcome = run_global_supervisor_boundary_review_core(
            &path,
            &request,
            canonical,
            |_root, _prompt| Ok(GOOD_BOUNDARY.to_string()),
        );
        assert_eq!(outcome.status, "ready", "{:?}", outcome.reason);
        let review = outcome.review.expect("ready");
        assert_eq!(review.project_id, canonical);
        assert_ne!(review.project_id, crate::project_id(root));
        let (store, _) = global_supervisor_review_store::load_store_soft(&path, 9_000);
        assert_eq!(store.boundary_reviews.len(), 1);
        assert_eq!(store.boundary_reviews[0].project_id, canonical);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_supervisor_official_resolve_unavailable_and_missing_alias() {
        let missing = resolve_global_supervisor_canonical_project_id(None, "/tmp/any-root");
        assert_eq!(
            missing.unwrap_err(),
            crate::m1_project_index::M1_PROJECT_INDEX_UNAVAILABLE
        );
        let stub = ResolveExactAliasStub {
            missing_code: "m1_alias_unknown",
        };
        let unknown = resolve_global_supervisor_canonical_project_id(Some(&stub), "/tmp/any-root");
        assert_eq!(unknown.unwrap_err(), "m1_alias_unknown");
        let empty = resolve_global_supervisor_canonical_project_id(Some(&stub), "");
        assert_eq!(empty.unwrap_err(), "m1_alias_malformed");
    }

    #[test]
    fn global_supervisor_production_does_not_mint_path_derived_project_id() {
        let source = include_str!("global_supervisor_agent.rs");
        let production_end = source.find("#[cfg(test)]\nmod tests").expect("test module");
        let production = &source[..production_end];
        assert!(production.contains("resolve_global_supervisor_canonical_project_id"));
        assert!(production.contains("m1_project_index_read_port"));
        assert!(production.contains("resolve_exact_alias"));
        assert!(production.contains("canonical_project_id"));
        assert!(!production.contains("crate::project_id(project_root)"));
        assert!(!production.contains("crate::project_id(&request.project_root)"));
    }
}
