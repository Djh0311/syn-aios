// 工作历史·后端读模型（按单列史·**纯只读**）。
//
// 任务包：tasks/2026-07-08-run-history-read-model-backend-v1.md
//
// 定位：把散在各店的「一单交办的一生」拼成一条历史记录。UI 长相由整台原型 M1 拍板后另出；
// 本模块与 UI 形态无关、先行落地。**零写入·零状态迁移·零 LM·零审计**（读模型不留痕）。
//
// 数据事实（主导线核·设计以此为准）：
//   · 方案 = project_consultation_proposal_store（proposal_id / status / 目标 / created_at_ms / scope_draft 写根空=纯建议）；
//   · 链   = workflow state 内 `workflow_chain_runs`（started_at / state / nodes 进度）——**没存 proposal_id**；
//   · 意见 = global_supervisor_review_store（结果复核按 (workflow_id, chain_started_at)、边界意见按 proposal_id）。
//
// 跨店没有外键：**链↔方案只能按 workflow_id + 时间窗诚实近似**（链归属「其 started_at 之前最近的已确认方案」）。
// 边界意见按 proposal_id 精确挂、结果复核按 (workflow_id, chain_started_at) 精确挂链再随链归单。
// `correlation` 字段如实标 exact / time_window；歧义（确认时间同毫秒级）→ 归最近者 + note 注明。
// **红线**：不为了好关联去改任何写入路径加字段——本模块只调现成只读 loader，近似就老实近似并在字段里注明。
//
// 注（回交已报备）：plan-authorizations store 实际带 `source_proposal_id`（线上 17/17 已填），
// 即 授权↔方案 本可精确；但**链**既无 proposal_id 也无 authorization_id，故 链↔方案 仍只能时间窗——
// 本模块按方案为脊、链时间窗归单，未依赖 auth store（六态无需它）。

use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

/// 一「单」的历史条目（字段全加法思维·UI 半包直接接）。
///
/// `state` 用稳定机器键（UI 自己映射人话/配色），`state_note` 才是人话一句：
/// - "pending"（待批）/ "advice_only"（纯建议·写根空）/ "confirmed_not_run"（批了没跑）/
///   "running"（跑着）/ "blocked"（卡住）/ "delivered"（交货）/
///   "declined"（已回绝）/ "superseded"（被替代）/ "changes_requested"（要改）。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunHistoryEntry {
    pub(crate) proposal_id: String,
    pub(crate) workflow_id: String,
    pub(crate) goal_text: String,
    pub(crate) created_at_ms: i64,
    pub(crate) state: String,
    pub(crate) state_note: String,
    pub(crate) advice_only: bool,
    pub(crate) chain: Option<RunHistoryChain>,
    pub(crate) review_flags: RunHistoryReviewFlags,
    /// "exact"（纯方案字段推导·无近似）| "time_window"（涉及链时间窗归属·可能错配）。
    pub(crate) correlation: String,
    /// A·运行错误人话（仅失败/中断态填·`{family, human, raw_snippet}`）：默认脸显 human、下钻看 raw_snippet。
    /// **纯呈现·不驱动**：不改 state/state_note/成败判定，只补一个可选诊断字段（延续 fix8「只影响报告」）。
    pub(crate) error: Option<crate::run_error_translation::RunErrorHuman>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunHistoryChain {
    pub(crate) started_at: String,
    pub(crate) done_count: usize,
    pub(crate) total_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(crate) struct RunHistoryReviewFlags {
    /// 结果复核总判（"pass"|"needs_rework"|"needs_human_check"）——随链时间窗归单。
    pub(crate) result_verdict: Option<String>,
    /// 批前边界意见（"looks_ok"|"mismatch"|"caution"）——按 proposal_id 精确挂。
    pub(crate) boundary_verdict: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunHistoryList {
    pub(crate) entries: Vec<RunHistoryEntry>,
    /// limit 前的总单数（前端翻页/"共 N 单"用）。
    pub(crate) total: usize,
    /// 软着陆报备：某店缺失/损坏时的人话（不 Err 断面板·增益不是闸）。
    pub(crate) warnings: Vec<String>,
}

const DEFAULT_LIMIT: usize = 50;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

// ===== 中间投影（把各店读成最小结构·assemble 只吃这些·纯逻辑可测） =====

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProposalStatusLite {
    Pending,
    Confirmed,
    Rejected,
    Superseded,
    Expired,
    ChangesRequested,
}

#[derive(Debug, Clone)]
struct ProposalLite {
    proposal_id: String,
    workflow_id: String,
    goal_text: String,
    status: ProposalStatusLite,
    advice_only: bool,
    created_at_ms: i64,
}

#[derive(Debug, Clone)]
struct ChainLite {
    workflow_id: String,
    started_at_str: String,
    started_at_ms: i64,
    state: String, // running / completed / failed / stopped / superseded
    done_count: usize,
    total_count: usize,
    /// A·失败/中断链的原始错误串（失败节点 message + 关联 dispatch failure_reason 合并·翻译前的原料）；
    /// completed/running/无失败信息 → None。只读投影·不改任何写入路径。
    failure_raw: Option<String>,
}

/// 首行 + 按 char 边界截断（防目标文本爆条目）。
fn first_line_clip(text: &str, max_chars: usize) -> String {
    let first = text.lines().next().unwrap_or("").trim();
    let chars: Vec<char> = first.chars().collect();
    if chars.len() <= max_chars {
        first.to_string()
    } else {
        format!("{}…", chars[..max_chars].iter().collect::<String>())
    }
}

/// 链归属：某链归「其 started_at 之前 created_at 最近的已确认方案」（同 workflow·时间窗）。
/// 返回 (owner_proposal_id, ambiguous)：ambiguous = 顶端两方案 created_at 同毫秒（归属真歧义）。
fn owner_for_chain(chain: &ChainLite, proposals: &[ProposalLite]) -> Option<(String, bool)> {
    let mut candidates: Vec<&ProposalLite> = proposals
        .iter()
        .filter(|proposal| {
            proposal.status == ProposalStatusLite::Confirmed
                && proposal.workflow_id == chain.workflow_id
                && proposal.created_at_ms <= chain.started_at_ms
        })
        .collect();
    // 最近者 = created_at 最大；歧义看并列。
    candidates.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    let top = candidates.first()?;
    let ambiguous = candidates
        .get(1)
        .map(|second| second.created_at_ms == top.created_at_ms)
        .unwrap_or(false);
    Some((top.proposal_id.clone(), ambiguous))
}

/// 纯逻辑装配（不碰盘·可测）：方案为脊、链时间窗归单、意见精确挂 → 按 created_at 倒序 + limit + total。
fn assemble(
    proposals: &[ProposalLite],
    chains: &[ChainLite],
    boundary_verdicts: &HashMap<String, String>,
    // (workflow_id, chain_started_at) → (overall_verdict, issue_count)
    result_reviews: &HashMap<(String, String), (String, usize)>,
    now_ms: i64,
    limit: usize,
) -> (Vec<RunHistoryEntry>, usize) {
    // 1. 链归单（每链算一个 owner + 歧义标记）。
    let mut chains_by_owner: HashMap<String, Vec<&ChainLite>> = HashMap::new();
    let mut ambiguous_owner: HashMap<String, bool> = HashMap::new();
    for chain in chains {
        if let Some((owner, ambiguous)) = owner_for_chain(chain, proposals) {
            chains_by_owner
                .entry(owner.clone())
                .or_default()
                .push(chain);
            let slot = ambiguous_owner.entry(owner).or_insert(false);
            *slot = *slot || ambiguous;
        }
    }

    // 2. 每方案一条目。
    let mut entries: Vec<RunHistoryEntry> = proposals
        .iter()
        .map(|proposal| {
            // 本方案的链：时间窗归来的，取 started_at 最新一条为「最新态」。
            let latest_chain = chains_by_owner
                .get(&proposal.proposal_id)
                .and_then(|owned| {
                    owned
                        .iter()
                        .max_by_key(|chain| chain.started_at_ms)
                        .copied()
                });
            let ambiguous = *ambiguous_owner.get(&proposal.proposal_id).unwrap_or(&false);

            let boundary_verdict = boundary_verdicts.get(&proposal.proposal_id).cloned();
            let result_verdict = latest_chain.and_then(|chain| {
                result_reviews
                    .get(&(proposal.workflow_id.clone(), chain.started_at_str.clone()))
                    .map(|(overall, _)| overall.clone())
            });
            let issue_count = latest_chain
                .and_then(|chain| {
                    result_reviews
                        .get(&(proposal.workflow_id.clone(), chain.started_at_str.clone()))
                })
                .map(|(_, issues)| *issues)
                .unwrap_or(0);

            let (state, mut state_note, correlation) =
                derive_state(proposal, latest_chain, now_ms, &result_verdict, issue_count);
            if latest_chain.is_some() && ambiguous {
                state_note = append_note(state_note, "归属按时间近似");
            }

            // A·运行错误人话：仅 blocked（失败/中断）态、且链带原始错误串时翻译；否则 None（不呈现）。
            let error = if state == "blocked" {
                latest_chain
                    .and_then(|chain| chain.failure_raw.as_deref())
                    .map(crate::run_error_translation::classify_run_error)
            } else {
                None
            };

            RunHistoryEntry {
                proposal_id: proposal.proposal_id.clone(),
                workflow_id: proposal.workflow_id.clone(),
                goal_text: proposal.goal_text.clone(),
                created_at_ms: proposal.created_at_ms,
                state,
                state_note,
                advice_only: proposal.advice_only,
                chain: latest_chain.map(|chain| RunHistoryChain {
                    started_at: chain.started_at_str.clone(),
                    done_count: chain.done_count,
                    total_count: chain.total_count,
                }),
                review_flags: RunHistoryReviewFlags {
                    result_verdict,
                    boundary_verdict,
                },
                correlation: correlation.to_string(),
                error,
            }
        })
        .collect();

    // 3. 倒序（新单在前）+ total（limit 前）+ 截断。
    entries.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    let total = entries.len();
    entries.truncate(limit);
    (entries, total)
}

fn append_note(note: String, extra: &str) -> String {
    if note.trim().is_empty() {
        format!("（{extra}）")
    } else {
        format!("{note}（{extra}）")
    }
}

/// 确定性状态推导（一单恰一态·规则有序）。返回 (state_key, state_note, correlation)。
fn derive_state(
    proposal: &ProposalLite,
    latest_chain: Option<&ChainLite>,
    now_ms: i64,
    result_verdict: &Option<String>,
    issue_count: usize,
) -> (String, String, &'static str) {
    // 非 confirmed 的终态/未决态：纯方案字段推导（correlation=exact）。
    match proposal.status {
        ProposalStatusLite::Pending => {
            let age_days = ((now_ms - proposal.created_at_ms).max(0)) / DAY_MS;
            let note = if age_days >= 1 {
                format!("{age_days} 天前的旧方案，可能过期，建议重新说一遍")
            } else {
                "等你批".to_string()
            };
            return ("pending".to_string(), note, "exact");
        }
        ProposalStatusLite::Rejected => {
            return (
                "declined".to_string(),
                "你回绝了这份方案".to_string(),
                "exact",
            );
        }
        ProposalStatusLite::Superseded => {
            return (
                "superseded".to_string(),
                "被后来的重拆取代了".to_string(),
                "exact",
            );
        }
        ProposalStatusLite::Expired => {
            return (
                "expired".to_string(),
                "显式期限已到，方案已关闭".to_string(),
                "exact",
            );
        }
        ProposalStatusLite::ChangesRequested => {
            return (
                "changes_requested".to_string(),
                "你要求改，等新方案".to_string(),
                "exact",
            );
        }
        ProposalStatusLite::Confirmed => {}
    }

    // 纯建议（写根空）优先于「批了没跑」：写根空=没东西可跑，是纯建议不是没做。
    // advice_only 判据来自 scope_draft（精确）——correlation=exact。
    if proposal.advice_only && latest_chain.is_none() {
        return (
            "advice_only".to_string(),
            "纯建议方案（不会改任何文件），已收下".to_string(),
            "exact",
        );
    }

    match latest_chain {
        // 已确认但时间窗关联不到任何链 → 批了没跑（用户点名要看见的「没做」）。
        None => (
            "confirmed_not_run".to_string(),
            "批过了，但还没开跑".to_string(),
            "time_window",
        ),
        Some(chain) => match chain.state.as_str() {
            "running" => (
                "running".to_string(),
                format!("跑到 {}/{}", chain.done_count, chain.total_count),
                "time_window",
            ),
            "completed" => {
                let note = if issue_count > 0 {
                    format!("做完了，有 {issue_count} 项要看一眼")
                } else if matches!(
                    result_verdict.as_deref(),
                    Some("needs_rework") | Some("needs_human_check")
                ) {
                    "做完了，全局主管建议你再看一眼".to_string()
                } else {
                    "做完了".to_string()
                };
                ("delivered".to_string(), note, "time_window")
            }
            other => {
                // failed / stopped / superseded / 其它 → 卡住（中断的）。
                // §3 不露黑话：**不透传节点原始 message**（线上实见 "real_execution_gate_blocked:…" 等黑话），
                // 只给人话状态尾巴；具体停因在「工作流」详情看（UI 半包接）。
                let note = match other {
                    "stopped" => "被手动停了（去工作流看详情）",
                    "superseded" => "旧链已被重拆取代",
                    "failed" => "跑挂了（去工作流看详情）",
                    _ => "跑中断了（去工作流看详情）",
                };
                ("blocked".to_string(), note.to_string(), "time_window")
            }
        },
    }
}

// ===== 读盘装配（软着陆·只调现成只读 loader） =====

/// 读各店（软着陆）→ 投影 → assemble。任一店缺失/损坏 → 该店数据缺席、其余照拼、不 Err。
pub(crate) fn list_project_run_history_at(
    state_path: &Path,
    project_root: &str,
    workflow_filter: Option<&str>,
    limit: usize,
) -> RunHistoryList {
    let now_ms = crate::unix_timestamp_ms();
    let pid = crate::project_id(project_root);
    let mut warnings: Vec<String> = Vec::new();

    // 1. 方案（脊）。
    let proposals: Vec<ProposalLite> =
        match crate::project_consultation_proposal_store::load_store(state_path, now_ms) {
            Ok(store) => store
                .proposals
                .iter()
                .filter(|proposal| {
                    proposal.project_id == pid
                        && workflow_filter
                            .map(|wid| proposal.workflow_id == wid)
                            .unwrap_or(true)
                })
                .map(|proposal| ProposalLite {
                    proposal_id: proposal.proposal_id.clone(),
                    workflow_id: proposal.workflow_id.clone(),
                    goal_text: first_line_clip(&proposal.user_goal, 80),
                    status: map_status(&proposal.status),
                    advice_only: proposal.scope_draft.allowed_write_roots.is_empty(),
                    created_at_ms: proposal.created_at_ms,
                })
                .collect(),
            Err(error) => {
                warnings.push(format!("方案库读不到（这部分历史缺席）：{error}"));
                Vec::new()
            }
        };

    // 2. 链（workflow state 内·现成只读口）。文件不存在=空项目（不算 warning）；存在但读坏=报备。
    let chains: Vec<ChainLite> = if !state_path.exists() {
        Vec::new()
    } else {
        match crate::read_workflow_state_value(state_path) {
            Ok(value) => project_chains(&value, &pid, workflow_filter),
            Err(error) => {
                warnings.push(format!("工作流状态读不到（跑记录这部分缺席）：{error}"));
                Vec::new()
            }
        }
    };

    // 3. 两道意见（软着陆 loader 自带 warnings）。
    let (boundary_verdicts, result_reviews, review_warnings) = load_review_maps(state_path, now_ms);
    warnings.extend(review_warnings);

    let (entries, total) = assemble(
        &proposals,
        &chains,
        &boundary_verdicts,
        &result_reviews,
        now_ms,
        limit,
    );
    RunHistoryList {
        entries,
        total,
        warnings,
    }
}

fn map_status(status: &crate::ProjectConsultationProposalStatus) -> ProposalStatusLite {
    match status {
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
        | crate::ProjectConsultationProposalStatus::Draft => ProposalStatusLite::Pending,
        crate::ProjectConsultationProposalStatus::UserConfirmed => ProposalStatusLite::Confirmed,
        crate::ProjectConsultationProposalStatus::ChangesRequested => {
            ProposalStatusLite::ChangesRequested
        }
        crate::ProjectConsultationProposalStatus::Rejected => ProposalStatusLite::Rejected,
        crate::ProjectConsultationProposalStatus::Superseded => ProposalStatusLite::Superseded,
        crate::ProjectConsultationProposalStatus::Expired => ProposalStatusLite::Expired,
    }
}

/// 从 workflow state value 投影 `workflow_chain_runs`（过滤 project + 可选 workflow）。
fn project_chains(
    value: &serde_json::Value,
    pid: &str,
    workflow_filter: Option<&str>,
) -> Vec<ChainLite> {
    value
        .get("workflow_chain_runs")
        .and_then(serde_json::Value::as_array)
        .map(|runs| {
            runs.iter()
                .filter(|run| {
                    crate::optional_string_from(run, "project_id").as_deref() == Some(pid)
                        && workflow_filter
                            .map(|wid| {
                                crate::optional_string_from(run, "workflow_id").as_deref()
                                    == Some(wid)
                            })
                            .unwrap_or(true)
                })
                .filter_map(|run| {
                    let started_at_str = crate::optional_string_from(run, "started_at")?;
                    let started_at_ms = started_at_str.parse::<i64>().ok()?;
                    let nodes = run.get("nodes").and_then(serde_json::Value::as_array);
                    let total_count = nodes.map(|n| n.len()).unwrap_or(0);
                    let done_count = nodes
                        .map(|n| {
                            n.iter()
                                .filter(|node| {
                                    crate::optional_string_from(node, "state").as_deref()
                                        == Some("completed")
                                })
                                .count()
                        })
                        .unwrap_or(0);
                    Some(ChainLite {
                        workflow_id: crate::optional_string_from(run, "workflow_id")
                            .unwrap_or_default(),
                        started_at_str,
                        started_at_ms,
                        state: crate::optional_string_from(run, "state").unwrap_or_default(),
                        done_count,
                        total_count,
                        failure_raw: chain_failure_raw(run, nodes, value),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A·抠一条失败/中断链的原始错误原料：失败节点 `message`（Err 路=原始错误）+ 关联 dispatch
/// `failure_reason`（exit≠0 路更丰富·含压缩 stderr）合并。都没有 → None。**纯只读**（不改写入路径）。
fn chain_failure_raw(
    run: &serde_json::Value,
    nodes: Option<&Vec<serde_json::Value>>,
    value: &serde_json::Value,
) -> Option<String> {
    // 只对非正常结束的链抠错误（running/completed 不抠）。
    let state = crate::optional_string_from(run, "state").unwrap_or_default();
    if state == "running" || state == "completed" {
        return None;
    }
    let nodes = nodes?;
    // 首个非完成节点：取它的 message + 关联 dispatch 的 failure_reason。
    let failed_node = nodes.iter().find(|node| {
        !matches!(
            crate::optional_string_from(node, "state").as_deref(),
            Some("completed") | Some("pending") | Some("skipped")
        )
    })?;
    let mut parts: Vec<String> = Vec::new();
    if let Some(message) = crate::optional_string_from(failed_node, "message") {
        if !message.trim().is_empty() {
            parts.push(message);
        }
    }
    if let Some(dispatch_id) = crate::optional_string_from(failed_node, "dispatch_id") {
        if let Some(dispatch) = value
            .get("workflow_node_dispatches")
            .and_then(serde_json::Value::as_array)
            .and_then(|dispatches| {
                dispatches.iter().find(|dispatch| {
                    crate::optional_string_from(dispatch, "dispatch_id").as_deref()
                        == Some(dispatch_id.as_str())
                })
            })
        {
            if let Some(reason) = crate::optional_string_from(dispatch, "failure_reason") {
                if !reason.trim().is_empty() && !parts.iter().any(|part| part == &reason) {
                    parts.push(reason);
                }
            }
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" ｜ "))
    }
}

/// 读 global_supervisor_review_store（软着陆）→ 两张挂接表。
#[allow(clippy::type_complexity)]
fn load_review_maps(
    state_path: &Path,
    now_ms: i64,
) -> (
    HashMap<String, String>,
    HashMap<(String, String), (String, usize)>,
    Vec<String>,
) {
    let (store, warnings) =
        crate::global_supervisor_review_store::load_store_soft(state_path, now_ms);
    let boundary_verdicts: HashMap<String, String> = store
        .boundary_reviews
        .iter()
        .filter(|review| review.status == "ready")
        .map(|review| (review.proposal_id.clone(), review.verdict.clone()))
        .collect();
    let result_reviews: HashMap<(String, String), (String, usize)> = store
        .reviews
        .iter()
        .filter(|review| review.status == "ready")
        .map(|review| {
            let issues = review
                .tasks
                .iter()
                .filter(|task| task.verdict == "issue")
                .count();
            (
                (review.workflow_id.clone(), review.chain_started_at.clone()),
                (review.overall.clone(), issues),
            )
        })
        .collect();
    (boundary_verdicts, result_reviews, warnings)
}

// ===== tauri 命令 =====

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ListProjectRunHistoryRequest {
    pub(crate) project_root: String,
    #[serde(default)]
    pub(crate) workflow_id: Option<String>,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
}

#[tauri::command]
pub(crate) fn list_project_run_history(
    request: ListProjectRunHistoryRequest,
    state: tauri::State<'_, crate::AppState>,
) -> RunHistoryList {
    // 纯只读·同步（读盘装配快·无 LM·无需 spawn_blocking）。任何缺失/损坏软着陆·不 Err 断面板。
    let limit = request.limit.unwrap_or(DEFAULT_LIMIT).max(1);
    list_project_run_history_at(
        &state.workflow_state_path,
        &request.project_root,
        request.workflow_id.as_deref(),
        limit,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn proposal(
        id: &str,
        wf: &str,
        status: ProposalStatusLite,
        advice_only: bool,
        created: i64,
    ) -> ProposalLite {
        ProposalLite {
            proposal_id: id.to_string(),
            workflow_id: wf.to_string(),
            goal_text: format!("目标 {id}"),
            status,
            advice_only,
            created_at_ms: created,
        }
    }

    fn chain(wf: &str, started: i64, state: &str, done: usize, total: usize) -> ChainLite {
        chain_with_error(wf, started, state, done, total, None)
    }

    fn chain_with_error(
        wf: &str,
        started: i64,
        state: &str,
        done: usize,
        total: usize,
        failure_raw: Option<&str>,
    ) -> ChainLite {
        ChainLite {
            workflow_id: wf.to_string(),
            started_at_str: started.to_string(),
            started_at_ms: started,
            state: state.to_string(),
            done_count: done,
            total_count: total,
            failure_raw: failure_raw.map(str::to_string),
        }
    }

    fn find<'a>(entries: &'a [RunHistoryEntry], id: &str) -> &'a RunHistoryEntry {
        entries
            .iter()
            .find(|entry| entry.proposal_id == id)
            .unwrap_or_else(|| panic!("entry {id} 应在"))
    }

    const NOW: i64 = 10_000_000_000;

    // §4：六态各一例（含批了没跑与纯建议）。
    #[test]
    fn six_states_each_covered() {
        let proposals = vec![
            proposal(
                "p-pending",
                "wf",
                ProposalStatusLite::Pending,
                false,
                NOW - 100,
            ),
            proposal("p-advice", "wf", ProposalStatusLite::Confirmed, true, 1_000),
            proposal("p-noRun", "wf", ProposalStatusLite::Confirmed, false, 2_000),
            proposal("p-run", "wf", ProposalStatusLite::Confirmed, false, 3_000),
            proposal(
                "p-blocked",
                "wf",
                ProposalStatusLite::Confirmed,
                false,
                4_000,
            ),
            proposal("p-done", "wf", ProposalStatusLite::Confirmed, false, 5_000),
        ];
        // 链只归到 run/blocked/done 三份（noRun 后于 3000 之前无链落在它窗内之外）。
        let chains = vec![
            chain("wf", 3_100, "running", 1, 3),   // 归 p-run（3000 最近）
            chain("wf", 4_100, "failed", 1, 2),    // 归 p-blocked（4000 最近）
            chain("wf", 5_100, "completed", 2, 2), // 归 p-done（5000 最近）
        ];
        let (entries, total) = assemble(
            &proposals,
            &chains,
            &HashMap::new(),
            &HashMap::new(),
            NOW,
            50,
        );
        assert_eq!(total, 6);
        assert_eq!(find(&entries, "p-pending").state, "pending");
        assert_eq!(find(&entries, "p-advice").state, "advice_only");
        assert_eq!(find(&entries, "p-noRun").state, "confirmed_not_run");
        assert_eq!(find(&entries, "p-run").state, "running");
        assert_eq!(find(&entries, "p-blocked").state, "blocked");
        assert_eq!(find(&entries, "p-done").state, "delivered");
        // 批了没跑：correlation=time_window（负向时间窗·可能错配）。
        assert_eq!(find(&entries, "p-noRun").correlation, "time_window");
        assert!(find(&entries, "p-noRun").state_note.contains("还没开跑"));
        // 纯建议：correlation=exact（scope_draft 精确）+ chain None。
        assert_eq!(find(&entries, "p-advice").correlation, "exact");
        assert!(find(&entries, "p-advice").chain.is_none());
        // 跑着带 done/total。
        let run = find(&entries, "p-run");
        assert_eq!(run.chain.as_ref().unwrap().done_count, 1);
        assert_eq!(run.chain.as_ref().unwrap().total_count, 3);
    }

    // §4：非 confirmed 终态/未决态各态（declined/superseded/changes_requested/draft→pending）。
    #[test]
    fn terminal_and_undecided_states() {
        let proposals = vec![
            proposal("p-rej", "wf", ProposalStatusLite::Rejected, false, 1_000),
            proposal("p-sup", "wf", ProposalStatusLite::Superseded, false, 2_000),
            proposal(
                "p-chg",
                "wf",
                ProposalStatusLite::ChangesRequested,
                false,
                3_000,
            ),
        ];
        let (entries, _) = assemble(&proposals, &[], &HashMap::new(), &HashMap::new(), NOW, 50);
        assert_eq!(find(&entries, "p-rej").state, "declined");
        assert_eq!(find(&entries, "p-sup").state, "superseded");
        assert_eq!(find(&entries, "p-chg").state, "changes_requested");
        // 全 exact（纯方案字段·无链）。
        for id in ["p-rej", "p-sup", "p-chg"] {
            assert_eq!(find(&entries, id).correlation, "exact");
        }
    }

    // §4：时间窗归属（两方案两链归对）+ 歧义标注。
    #[test]
    fn time_window_ownership_and_ambiguity() {
        // A 确认 1000，B 确认 2000；链 1500 归 A、链 2500 归 B。
        let proposals = vec![
            proposal("A", "wf", ProposalStatusLite::Confirmed, false, 1_000),
            proposal("B", "wf", ProposalStatusLite::Confirmed, false, 2_000),
        ];
        let chains = vec![
            chain("wf", 1_500, "completed", 1, 1),
            chain("wf", 2_500, "running", 0, 2),
        ];
        let (entries, _) = assemble(
            &proposals,
            &chains,
            &HashMap::new(),
            &HashMap::new(),
            NOW,
            50,
        );
        assert_eq!(find(&entries, "A").state, "delivered", "1500 链归 A");
        assert_eq!(find(&entries, "B").state, "running", "2500 链归 B");
        assert!(!find(&entries, "A").state_note.contains("归属按时间近似"));

        // 歧义：两方案 created_at 同毫秒（1000），链 1500 → 归最近者 + 注明。
        let tie = vec![
            proposal("X", "wf", ProposalStatusLite::Confirmed, false, 1_000),
            proposal("Y", "wf", ProposalStatusLite::Confirmed, false, 1_000),
        ];
        let (entries2, _) = assemble(
            &tie,
            &[chain("wf", 1_500, "completed", 1, 1)],
            &HashMap::new(),
            &HashMap::new(),
            NOW,
            50,
        );
        // 恰一条目拿到链（歧义归最近者·并列时 sort 稳定取其一），且带近似注。
        let with_chain: Vec<&RunHistoryEntry> = entries2
            .iter()
            .filter(|entry| entry.chain.is_some())
            .collect();
        assert_eq!(with_chain.len(), 1, "歧义也只归一份·不重复挂链");
        assert!(
            with_chain[0].state_note.contains("归属按时间近似"),
            "歧义应注明：{}",
            with_chain[0].state_note
        );
    }

    // §4：交货 + 复核 issue → 「有 N 项要看」；无复核不硬造。
    #[test]
    fn delivered_with_and_without_review() {
        let proposals = vec![
            proposal("p-issue", "wf", ProposalStatusLite::Confirmed, false, 1_000),
            proposal("p-clean", "wf", ProposalStatusLite::Confirmed, false, 2_000),
        ];
        let chains = vec![
            chain("wf", 1_500, "completed", 2, 2),
            chain("wf", 2_500, "completed", 2, 2),
        ];
        let mut results: HashMap<(String, String), (String, usize)> = HashMap::new();
        results.insert(
            ("wf".to_string(), "1500".to_string()),
            ("needs_human_check".to_string(), 2),
        );
        // p-clean（2500 链）无复核记录 → 不硬造。
        let (entries, _) = assemble(&proposals, &chains, &HashMap::new(), &results, NOW, 50);
        let issue = find(&entries, "p-issue");
        assert_eq!(issue.state, "delivered");
        assert!(
            issue.state_note.contains("有 2 项要看"),
            "{}",
            issue.state_note
        );
        assert_eq!(
            issue.review_flags.result_verdict.as_deref(),
            Some("needs_human_check")
        );
        let clean = find(&entries, "p-clean");
        assert_eq!(clean.state_note, "做完了", "无复核不硬造");
        assert!(clean.review_flags.result_verdict.is_none());
    }

    // A·§4 接线：失败链 → entry.error 投影出 {人话/原文/族}；state/state_note 逐字节不变（呈现纯增·不驱动）。
    #[test]
    fn failed_run_projects_translated_error_without_touching_state() {
        let proposals = vec![proposal(
            "p-fail",
            "wf",
            ProposalStatusLite::Confirmed,
            false,
            1_000,
        )];
        // 失败链带子系统原始错误（07-08 活证据形态）。
        let chains = vec![chain_with_error(
            "wf",
            1_500,
            "failed",
            0,
            1,
            Some("codex_memories_write::phase2::job: failed to claim job (no such table: jobs)"),
        )];
        let (entries, _) = assemble(
            &proposals,
            &chains,
            &HashMap::new(),
            &HashMap::new(),
            NOW,
            50,
        );
        let entry = find(&entries, "p-fail");
        // 成败呈现字段不变（延续既有 blocked 语义·A 不驱动）。
        assert_eq!(entry.state, "blocked");
        assert!(
            entry.state_note.contains("跑挂了"),
            "state_note 不变：{}",
            entry.state_note
        );
        // A 新增诊断字段：翻成人话 + 带原文 + 族。
        let error = entry.error.as_ref().expect("失败单应有翻译错误");
        assert_eq!(error.family, "codex_subsystem", "no such table → 子系统族");
        assert!(
            error.human.contains("子系统"),
            "人话不是裸错误：{}",
            error.human
        );
        assert!(!error.human.contains("no such table"), "默认脸不灌原文");
        assert!(error.raw_snippet.contains("no such table"), "下钻原文保留");
    }

    // A·§4：完成/跑中的单 error=None（只失败态呈现·不误挂）。
    #[test]
    fn non_blocked_runs_have_no_error() {
        let proposals = vec![
            proposal("p-done", "wf", ProposalStatusLite::Confirmed, false, 1_000),
            proposal("p-run", "wf2", ProposalStatusLite::Confirmed, false, 2_000),
        ];
        let chains = vec![
            chain_with_error("wf", 1_500, "completed", 1, 1, None),
            // 跑中链即便带 failure_raw（防御性），completed/running 也不抠——这里直接给 running。
            chain_with_error("wf2", 2_500, "running", 0, 2, None),
        ];
        let (entries, _) = assemble(
            &proposals,
            &chains,
            &HashMap::new(),
            &HashMap::new(),
            NOW,
            50,
        );
        assert!(find(&entries, "p-done").error.is_none(), "交货单无 error");
        assert!(find(&entries, "p-run").error.is_none(), "跑中单无 error");
    }

    // A·§4：project_chains 只对失败/中断态抠 failure_raw（completed/running 不抠·纯只读投影正确）。
    #[test]
    fn project_chains_extracts_failure_raw_only_for_terminal_failures() {
        let value = serde_json::json!({
            "workflow_chain_runs": [
                {
                    "project_id": "proj", "workflow_id": "wf", "state": "failed", "started_at": "1500",
                    "nodes": [{"node_id": "n1", "state": "failed", "dispatch_id": "d1",
                               "message": "worker 派发未完成（state=failed）"}]
                },
                {
                    "project_id": "proj", "workflow_id": "wf2", "state": "completed", "started_at": "2500",
                    "nodes": [{"node_id": "n2", "state": "completed"}]
                }
            ],
            "workflow_node_dispatches": [
                {"dispatch_id": "d1", "failure_reason": "codex_resume_exit_nonzero, attempt to write a readonly database"}
            ]
        });
        let chains = project_chains(&value, "proj", None);
        let failed = chains
            .iter()
            .find(|c| c.workflow_id == "wf")
            .expect("failed 链在");
        let raw = failed.failure_raw.as_deref().expect("失败链抠出原料");
        assert!(raw.contains("worker 派发未完成"), "含节点 message");
        assert!(
            raw.contains("readonly database"),
            "含 dispatch failure_reason（更丰富原文）"
        );
        let done = chains
            .iter()
            .find(|c| c.workflow_id == "wf2")
            .expect("完成链在");
        assert!(done.failure_raw.is_none(), "completed 不抠 failure_raw");
    }

    // §4：边界意见按 proposal_id 精确挂（exact·与链无关）。
    #[test]
    fn boundary_verdict_exact_by_proposal_id() {
        let proposals = vec![proposal(
            "p1",
            "wf",
            ProposalStatusLite::Pending,
            false,
            NOW - 10,
        )];
        let mut boundary: HashMap<String, String> = HashMap::new();
        boundary.insert("p1".to_string(), "mismatch".to_string());
        let (entries, _) = assemble(&proposals, &[], &boundary, &HashMap::new(), NOW, 50);
        assert_eq!(
            find(&entries, "p1")
                .review_flags
                .boundary_verdict
                .as_deref(),
            Some("mismatch")
        );
    }

    // §4：stale 旧方案 note（口径：>=1 天前）。
    #[test]
    fn stale_pending_note() {
        let fresh = vec![proposal(
            "f",
            "wf",
            ProposalStatusLite::Pending,
            false,
            NOW - 1_000,
        )];
        let (e1, _) = assemble(&fresh, &[], &HashMap::new(), &HashMap::new(), NOW, 50);
        assert_eq!(find(&e1, "f").state_note, "等你批");
        let old = vec![proposal(
            "o",
            "wf",
            ProposalStatusLite::Pending,
            false,
            NOW - 3 * DAY_MS,
        )];
        let (e2, _) = assemble(&old, &[], &HashMap::new(), &HashMap::new(), NOW, 50);
        assert!(
            find(&e2, "o").state_note.contains("3 天前"),
            "{}",
            find(&e2, "o").state_note
        );
    }

    // §4：倒序 + limit + total。
    #[test]
    fn order_limit_total() {
        let proposals: Vec<ProposalLite> = (0..5)
            .map(|i| {
                proposal(
                    &format!("p{i}"),
                    "wf",
                    ProposalStatusLite::Pending,
                    false,
                    1_000 + i as i64 * 1_000,
                )
            })
            .collect();
        let (entries, total) = assemble(&proposals, &[], &HashMap::new(), &HashMap::new(), NOW, 2);
        assert_eq!(total, 5, "total = limit 前总数");
        assert_eq!(entries.len(), 2, "limit 截断");
        // 倒序：created 最大在前。
        assert_eq!(entries[0].proposal_id, "p4");
        assert_eq!(entries[1].proposal_id, "p3");
    }

    // §4：纯建议即便「已确认」也判纯建议（写根空=没东西跑·规则序：advice 先于批了没跑）。
    #[test]
    fn confirmed_advice_only_is_advice_not_notrun() {
        let proposals = vec![proposal(
            "a",
            "wf",
            ProposalStatusLite::Confirmed,
            true,
            1_000,
        )];
        let (entries, _) = assemble(&proposals, &[], &HashMap::new(), &HashMap::new(), NOW, 50);
        assert_eq!(find(&entries, "a").state, "advice_only");
    }

    // ===== 读盘装配·软着陆（整店造夹具·照 B1/B2 手写先例） =====

    fn tmp_state(tag: &str) -> (PathBuf, PathBuf) {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("run-history-{tag}-{uniq}"));
        fs::create_dir_all(&dir).expect("tmp dir");
        (dir.clone(), dir.join("workflow-state.v0.json"))
    }

    fn write_proposals(state_path: &Path, pid: &str) {
        let sidecar =
            crate::project_consultation_proposal_store::sidecar_path(state_path).expect("sidecar");
        if let Some(parent) = sidecar.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mk = |id: &str, wr: serde_json::Value, status: &str, created: i64| {
            serde_json::json!({
                "proposal_id": id, "schema_version": "project_consultation_proposal.v1",
                "project_id": pid, "workflow_id": "wf-1", "title": "t",
                "user_goal": format!("目标 {id}"), "goal_summary": "s",
                "proposed_steps": ["a"],
                "scope_draft": {"allowed_role_ids": [], "allowed_agent_ids": [], "allowed_read_roots": [],
                    "allowed_write_roots": wr, "allowed_tools": [], "allowed_checks": [],
                    "allowed_task_package_kinds": [], "stop_conditions": [],
                    "max_worker_dispatches": null, "max_runtime_minutes": null},
                "risks": [], "acceptance_criteria": ["ok"], "status": status,
                "plan_authorization_id": null, "created_by_role": "project_consultant",
                "suggest_workflow": false, "created_at_ms": created, "updated_at_ms": created
            })
        };
        let store = serde_json::json!({
            "schema_version": "project_consultation_proposal_store.v1", "revision": 1,
            "proposals": [
                mk("p-done", serde_json::json!(["/t"]), "user_confirmed", 5_000),
                mk("p-advice", serde_json::json!([]), "user_confirmed", 1_000),
            ],
            "decisions": [], "audit_events": [], "updated_at_ms": 1, "warnings": []
        });
        fs::write(&sidecar, serde_json::to_string_pretty(&store).unwrap())
            .expect("write proposals");
    }

    fn write_workflow_state(state_path: &Path, pid: &str) {
        let store = serde_json::json!({
            "schema_version": "workflow_state_v0", "workflow_version": 1, "updated_at": "seed",
            "projects": [], "agent_adapters": [], "workflows": [{"workflow_id": "wf-1"}],
            "nodes": [], "edges": [], "work_items": [], "artifacts": [], "reviews": [],
            "audit_events": [], "capabilities": [], "harness_resources": [],
            "workflow_chain_runs": [{
                "chain_run_id": "c1", "project_id": pid, "workflow_id": "wf-1",
                "state": "completed", "stop_requested": false, "started_at": "5100", "ended_at": "5200",
                "nodes": [{"node_id": "n1", "state": "completed", "dispatch_id": null, "message": null}]
            }]
        });
        fs::write(state_path, serde_json::to_string_pretty(&store).unwrap()).expect("write state");
    }

    // §4：读盘整合 —— 真店造夹具 → 交货（p-done 关联 5100 链）+ 纯建议（p-advice）。
    #[test]
    fn load_integration_assembles_from_real_stores() {
        let (dir, state_path) = tmp_state("integ");
        let pid = crate::project_id("/proj/root");
        write_proposals(&state_path, &pid);
        write_workflow_state(&state_path, &pid);
        let out = list_project_run_history_at(&state_path, "/proj/root", None, 50);
        assert!(
            out.warnings.is_empty(),
            "干净店无 warning：{:?}",
            out.warnings
        );
        assert_eq!(out.total, 2);
        assert_eq!(find(&out.entries, "p-done").state, "delivered");
        assert_eq!(find(&out.entries, "p-advice").state, "advice_only");
        // 倒序：p-done(5000) 在 p-advice(1000) 前。
        assert_eq!(out.entries[0].proposal_id, "p-done");
        let _ = fs::remove_dir_all(dir);
    }

    // §4：店损坏软着陆（方案库坏 → 该部分缺席 + warning·不 Err·其余照拼）。
    #[test]
    fn corrupt_proposal_store_soft_lands() {
        let (dir, state_path) = tmp_state("corrupt");
        let pid = crate::project_id("/proj/root");
        let sidecar =
            crate::project_consultation_proposal_store::sidecar_path(&state_path).expect("sidecar");
        if let Some(parent) = sidecar.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&sidecar, "{ 坏 json").expect("write corrupt");
        write_workflow_state(&state_path, &pid);
        let out = list_project_run_history_at(&state_path, "/proj/root", None, 50);
        // 方案库坏 → 空脊 + warning，但不 Err、不 panic。
        assert!(out.entries.is_empty(), "无方案脊 → 空列表");
        assert!(
            out.warnings.iter().any(|w| w.contains("方案库读不到")),
            "应有软着陆 warning：{:?}",
            out.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4：空项目 → 空列表（文件全不存在·不 Err·无 warning）。
    #[test]
    fn empty_project_empty_list() {
        let (dir, state_path) = tmp_state("empty");
        let out = list_project_run_history_at(&state_path, "/nope/root", None, 50);
        assert!(out.entries.is_empty());
        assert_eq!(out.total, 0);
        let _ = fs::remove_dir_all(dir);
    }

    // §5·线上真店抽查（纯只读·无 LM·`#[ignore]`）：对固定测试项目跑读模型、打印每单归属供人核。
    // 显式 `cargo test --lib run_history_live_store_probe -- --ignored --nocapture`。
    #[test]
    #[ignore = "run-history: read-only probe of the live store (print per-run association for manual spot-check)"]
    fn run_history_live_store_probe() {
        let state_path = crate::default_workflow_state_path();
        if !state_path.exists() {
            println!("[RH_PROBE] 线上 store 不存在，跳过");
            return;
        }
        let out = list_project_run_history_at(
            &state_path,
            "/Users/yoyi/codex-workflow-mario-test",
            None,
            50,
        );
        println!("[RH_PROBE] total={} warnings={:?}", out.total, out.warnings);
        for entry in &out.entries {
            let chain = entry
                .chain
                .as_ref()
                .map(|c| format!("chain@{}({}/{})", c.started_at, c.done_count, c.total_count))
                .unwrap_or_else(|| "无链".to_string());
            println!(
                "[RH_PROBE] {} | {} | corr={} | {} | b={:?} r={:?} | {} | {}",
                entry.created_at_ms,
                entry.state,
                entry.correlation,
                chain,
                entry.review_flags.boundary_verdict,
                entry.review_flags.result_verdict,
                entry.state_note,
                entry.goal_text
            );
        }
        // 纯只读断言：读模型不写盘（跑前后 store mtime 不变）。
        let mtime_before = fs::metadata(&state_path).and_then(|m| m.modified()).ok();
        let _ = list_project_run_history_at(
            &state_path,
            "/Users/yoyi/codex-workflow-mario-test",
            None,
            50,
        );
        let mtime_after = fs::metadata(&state_path).and_then(|m| m.modified()).ok();
        assert_eq!(
            mtime_before, mtime_after,
            "读模型纯只读·不许写 workflow state"
        );
    }
}
