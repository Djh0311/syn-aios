// B3·秘书 agent（按需解释·唯一烧额度处）。
//
// 任务包：tasks/2026-07-07-phase-b3-secretary-pending-board-v1.md
// 决策正本：decisions/2026-07-07-phase-b-advisory-supervisor-and-secretary-v1.md 第 2 条
// 架构宪法：docs/workbench-system-architecture-v1.md §7 秘书核心协作层
//
// 安全属性（比 B1/B2 更严）：
// - 秘书**全程零写入**——没有自己的 store、不落解释、不写审计（解释是即抛的帮助不是记录；前端会话内缓存足够）；
// - **结构性只读**：LM 通道 = 现成 `readonly_codex_consult`（read-only 沙箱·写盘根空·只调不改）；
// - **输入全后端盘读**（不收前端转述文本）：pending 方案（proposal store）/ 主管两类意见（review store·soft）/
//   记忆候选计数（memory candidate store）——全走各店现成只读 loader；
// - 返回纯文本解释（无 json 契约·无解析步）；任何失败不 Err 断面板（status 带态·fix8 供给类人话）。

use serde::Serialize;

/// 秘书档案（prompt 头）。职责与禁区照抄架构 §7 原文——禁区是宪法，不是措辞。
const SECRETARY_PROFILE_TEXT: &str = "你是工作台的「秘书」。你的职责（架构 §7）：汇总项目和全局状态；整理待用户确认的建议；提醒权限、记忆、知识库和项目结构变化风险；帮用户理解多个智能体工作状态。你不能（§7 禁区原文）：绕过用户确认直接改系统事实；绕过项目主管直接操作项目；绕过权限读取项目私密资料；把聊天内容直接写入长期记忆；替代审计中心。你整理和解释，不判断、不裁决、不派活——所有事的最终决定权在用户。";

/// consult 走固定测试项目根当 cwd（readonly consult 需要一个真实存在的目录；秘书解释的是工作台
/// 全局状态、不读项目文件，cwd 只是形式锚——与只读咨询「可任意项目」先例一致取最稳的那个）。
const SECRETARY_CONSULT_ROOT: &str = "/Users/yoyi/codex-workflow-mario-test";
const SECRETARY_CONSULT_TIMEOUT_MS: i64 = 420_000;

/// 盘读出来的「待拍板事实」投影（喂 prompt 用·全部来自现成只读 loader）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SecretaryExplainFacts {
    /// 待批方案：「标题」列表（含相对天数标注）。
    pub(crate) pending_proposals: Vec<String>,
    /// 需留意的主管意见（结果复核 needs_human_check/human_verify + 批前 mismatch）。
    pub(crate) supervisor_reminders: Vec<String>,
    /// 待确认记忆候选条数。
    pub(crate) pending_memory_candidate_count: usize,
    /// 各店读取失败的人话注记（读不到就老实说读不到，不编）。
    pub(crate) read_warnings: Vec<String>,
}

/// 按 char 截断（喂 prompt 防爆）。
fn clip(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= max_chars {
        trimmed.to_string()
    } else {
        let head: String = chars[..max_chars].iter().collect();
        format!("{head}…")
    }
}

/// 盘读装配（每路 best-effort：某店读失败 → 该路空 + 人话注记，不 Err 整体）。
pub(crate) fn load_secretary_explain_facts(state_path: &std::path::Path) -> SecretaryExplainFacts {
    let timestamp_ms = crate::unix_timestamp_ms();
    let mut facts = SecretaryExplainFacts::default();
    // 1. 待批方案（proposal store·现成 loader）。
    match crate::project_consultation_proposal_store::load_store(state_path, timestamp_ms) {
        Ok(store) => {
            facts.pending_proposals = store
                .proposals
                .iter()
                .filter(|proposal| {
                    matches!(
                        proposal.status,
                        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
                    )
                })
                .map(|proposal| {
                    let age_days =
                        (timestamp_ms - proposal.created_at_ms).max(0) / (24 * 60 * 60 * 1000);
                    if age_days >= 1 {
                        format!(
                            "「{}」（{} 天前生成·偏旧）",
                            clip(&proposal.title, 80),
                            age_days
                        )
                    } else {
                        format!("「{}」（今天生成）", clip(&proposal.title, 80))
                    }
                })
                .collect();
        }
        Err(error) => facts.read_warnings.push(format!("方案店读不到（{error}）")),
    }
    // 2. 主管两类意见（review store·soft 语义损坏不炸）。
    let (review_store, review_warnings) =
        crate::global_supervisor_review_store::load_store_soft(state_path, timestamp_ms);
    facts.read_warnings.extend(review_warnings);
    for review in &review_store.reviews {
        if review.status == "ready"
            && (review.overall == "needs_human_check" || review.suggested_action == "human_verify")
        {
            let note = if review.human_note.trim().is_empty() {
                clip(&review.summary, 120)
            } else {
                clip(&review.human_note, 120)
            };
            facts
                .supervisor_reminders
                .push(format!("结果复核建议亲验：{note}"));
        }
    }
    for boundary in &review_store.boundary_reviews {
        if boundary.status == "ready" && boundary.verdict == "mismatch" {
            facts.supervisor_reminders.push(format!(
                "批前意见（方案与目标对不上）：{}",
                clip(&boundary.summary, 120)
            ));
        }
    }
    // 3. 记忆候选计数（memory candidate store·现成 loader）。
    match crate::memory_candidate_store::load_store(state_path, &crate::unix_timestamp_string()) {
        Ok(store) => {
            facts.pending_memory_candidate_count = store
                .candidates
                .iter()
                .filter(|candidate| {
                    matches!(
                        candidate.status,
                        crate::MemoryLifecycleStatus::CandidateDraft
                            | crate::MemoryLifecycleStatus::CandidateNeedsReview
                    )
                })
                .count();
        }
        Err(error) => facts
            .read_warnings
            .push(format!("记忆候选店读不到（{error}）")),
    }
    facts
}

/// 组 prompt（档案 + 盘读事实 + 要求·确定性拼接）。
pub(crate) fn build_secretary_explain_prompt(facts: &SecretaryExplainFacts) -> String {
    let mut sections = vec![SECRETARY_PROFILE_TEXT.to_string()];
    let mut board = String::from("【当前等用户拍板的事（工作台盘上事实）】");
    if facts.pending_proposals.is_empty() {
        board.push_str("\n- 待批方案：无");
    } else {
        board.push_str(&format!(
            "\n- 待批方案 {} 份：",
            facts.pending_proposals.len()
        ));
        for proposal in facts.pending_proposals.iter().take(8) {
            board.push_str(&format!("\n  · {proposal}"));
        }
    }
    if facts.supervisor_reminders.is_empty() {
        board.push_str("\n- 全局主管提醒：无");
    } else {
        board.push_str(&format!(
            "\n- 全局主管提醒 {} 条：",
            facts.supervisor_reminders.len()
        ));
        for reminder in facts.supervisor_reminders.iter().take(8) {
            board.push_str(&format!("\n  · {reminder}"));
        }
    }
    board.push_str(&format!(
        "\n- 待确认记忆候选：{} 条",
        facts.pending_memory_candidate_count
    ));
    if !facts.read_warnings.is_empty() {
        board.push_str("\n- 数据注记（读不到的店·老实说明）：");
        for warning in facts.read_warnings.iter().take(4) {
            board.push_str(&format!("\n  · {}", clip(warning, 160)));
        }
    }
    sections.push(board);
    sections.push(
        "【要求】用全中文人话把上面这份「等用户拍板的事」解释一遍：每件事是什么、为什么等用户、\
         建议先看哪件（只是建议顺序，不是命令）。简短（200 字内），别用内部黑话，别编造上面没有的事，\
         别说「审批」。直接输出解释文字，不要 json、不要代码块。"
            .to_string(),
    );
    sections.join("\n\n")
}

/// 返回结构（任何失败不 Err 断面板）。
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SecretaryExplainOutcome {
    /// "ready" | "unavailable"
    pub(crate) status: String,
    pub(crate) explanation: Option<String>,
    pub(crate) reason: Option<String>,
}

// A·收编（2026-07-09）：consult 错误翻人话改调单一真源 `run_error_translation`（原逐字节重复的
// humanize_consult_error 已删·供给前缀语义不变、非供给错误现也翻人话）。不动 director retry 读法。
fn humanize_consult_error(raw: &str) -> String {
    crate::run_error_translation::humanize_error_for_display(raw)
}

/// 核心（consult 可注入·单测 stub 验输入 grounded）。**零持久化**：解释即抛，不落任何盘。
pub(crate) fn run_secretary_explain_core<F>(
    state_path: &std::path::Path,
    consult: F,
) -> SecretaryExplainOutcome
where
    F: Fn(&str, &str) -> Result<String, String>,
{
    let facts = load_secretary_explain_facts(state_path);
    let prompt = build_secretary_explain_prompt(&facts);
    match consult(SECRETARY_CONSULT_ROOT, &prompt) {
        Ok(raw) => {
            let text = raw.trim().to_string();
            if text.is_empty() {
                SecretaryExplainOutcome {
                    status: "unavailable".to_string(),
                    explanation: None,
                    reason: Some("秘书这次没解释出来（回了空话），可以重试。".to_string()),
                }
            } else {
                SecretaryExplainOutcome {
                    status: "ready".to_string(),
                    explanation: Some(text),
                    reason: None,
                }
            }
        }
        Err(error) => SecretaryExplainOutcome {
            status: "unavailable".to_string(),
            explanation: None,
            reason: Some(humanize_consult_error(&error)),
        },
    }
}

#[tauri::command]
pub(crate) async fn run_secretary_explain(
    state: tauri::State<'_, crate::AppState>,
) -> Result<SecretaryExplainOutcome, String> {
    // 真 consult 长耗时 → spawn_blocking 不冻 UI（同咨询/复核范本）；path 在 await 前取。
    let path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_secretary_explain_core(&path, |project_root, prompt| {
            crate::codex_local_runner::readonly_codex_consult(
                project_root,
                prompt,
                Some(SECRETARY_CONSULT_TIMEOUT_MS),
            )
        })
    })
    .await
    .map_err(|error| format!("秘书解释执行线程异常：{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_dir(tag: &str) -> PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("secretary-{tag}-{uniq}"));
        fs::create_dir_all(&dir).expect("tmp dir");
        dir
    }

    fn fixture_project_record(project_root: &str) -> crate::ProjectRecord {
        crate::ProjectRecord {
            project_root: project_root.to_string(),
            name: "测试项目".to_string(),
            active_hint: true,
            thread_count: 0,
            active_thread_count: 0,
            archived_thread_count: 0,
            latest_updated_at_ms: None,
            authority_files: vec![],
            handoff_files: vec![],
            evidence_files: vec![],
            harness_candidates: vec![],
            harness_resources: vec![],
            context_warnings: vec![],
            warnings: vec![],
        }
    }

    /// 造盘上事实：1 份 pending 方案（真 proposal store API·先 bootstrap 真 state）+
    /// 1 条 needs_human_check 复核 + 1 条 mismatch 边界意见 + 1 条 caution（**不该进提醒**）。
    fn seed_stores(state_path: &std::path::Path) {
        crate::bootstrap_project_workflow_at(state_path, &fixture_project_record("/p/root"))
            .expect("bootstrap");
        let consult = crate::ConsultationProposal {
            user_goal: "把计分板改成暗色".to_string(),
            goal_summary: "计分板暗色主题".to_string(),
            scope_note: "改文件".to_string(),
            reasoning: vec!["r".to_string()],
            risks: vec![],
            must_stop_points: vec![],
            worker_acceptance_criteria: vec!["按方案完成执行步骤并返回证据".to_string()],
            control_core_acceptance_criteria: vec!["校验授权范围并记录状态".to_string()],
            supervisor_acceptance_criteria: vec!["检查回程证据后给出结论".to_string()],
            next_steps: vec!["改".to_string()],
            execution_scope: Some(crate::ConsultationExecutionScope {
                requires_write: true,
                write_roots: vec![],
                target_files: vec!["index.html".to_string()],
                tools: vec![],
                checks: vec![],
            }),
            suggest_workflow: false,
        };
        let c1 =
            crate::map_consultation_to_c1_input(&consult, "/p/root", "consultant").expect("map");
        crate::project_consultation_proposal_store::create_proposal(
            state_path,
            &c1,
            crate::unix_timestamp_ms(),
            "secretary-seed-proposal",
        )
        .expect("proposal");
        crate::global_supervisor_review_store::upsert_review(
            state_path,
            crate::global_supervisor_review_store::GlobalSupervisorReviewRecord {
                review_id: "r1".to_string(),
                workflow_id: "wf-1".to_string(),
                chain_started_at: "1000".to_string(),
                status: "ready".to_string(),
                overall: "needs_human_check".to_string(),
                suggested_action: "human_verify".to_string(),
                human_note: "打开 index.html 亲手玩一遍".to_string(),
                ..Default::default()
            },
            "seed",
            crate::unix_timestamp_ms(),
        )
        .expect("review");
        crate::global_supervisor_review_store::upsert_boundary_review(
            state_path,
            crate::global_supervisor_review_store::GlobalSupervisorBoundaryReviewRecord {
                review_id: "b1".to_string(),
                proposal_id: "p-mismatch".to_string(),
                status: "ready".to_string(),
                verdict: "mismatch".to_string(),
                summary: "方案只读不改文件，和用户要动手的目标对不上".to_string(),
                ..Default::default()
            },
            "seed",
            crate::unix_timestamp_ms(),
        )
        .expect("boundary");
        crate::global_supervisor_review_store::upsert_boundary_review(
            state_path,
            crate::global_supervisor_review_store::GlobalSupervisorBoundaryReviewRecord {
                review_id: "b2".to_string(),
                proposal_id: "p-caution".to_string(),
                status: "ready".to_string(),
                verdict: "caution".to_string(),
                summary: "验收偏薄（caution·不该进提醒）".to_string(),
                ..Default::default()
            },
            "seed",
            crate::unix_timestamp_ms(),
        )
        .expect("boundary caution");
    }

    // §4：explain 输入装配 grounded——stub consult 收到的 prompt 里有盘上 pending 事实；
    // caution 不进提醒；档案禁区原文在；返回纯文本。
    #[test]
    fn explain_prompt_grounded_from_disk_and_caution_excluded() {
        let dir = tmp_dir("grounded");
        let state_path = dir.join("workflow-state.v0.json");
        seed_stores(&state_path);
        let facts = load_secretary_explain_facts(&state_path);
        assert_eq!(facts.pending_proposals.len(), 1, "1 份 pending 方案");
        assert_eq!(
            facts.supervisor_reminders.len(),
            2,
            "needs_human_check + mismatch 各 1；caution 不进：{:?}",
            facts.supervisor_reminders
        );
        assert!(
            !facts
                .supervisor_reminders
                .iter()
                .any(|r| r.contains("caution") || r.contains("验收偏薄")),
            "caution 是提醒过的，别堆进秘书面"
        );
        let seen_prompt = RefCell::new(String::new());
        let consult = |_root: &str, prompt: &str| {
            *seen_prompt.borrow_mut() = prompt.to_string();
            Ok("现在有一份方案等你批，主管建议你亲验上一单。建议先看方案。".to_string())
        };
        let outcome = run_secretary_explain_core(&state_path, consult);
        assert_eq!(outcome.status, "ready");
        assert!(outcome.explanation.unwrap().contains("方案"));
        let prompt = seen_prompt.borrow();
        assert!(
            prompt.contains("计分板"),
            "prompt 应含盘上方案标题（grounded）"
        );
        assert!(prompt.contains("亲手玩一遍"), "prompt 应含复核 human_note");
        assert!(prompt.contains("对不上"), "prompt 应含 mismatch 摘要");
        assert!(
            prompt.contains("不判断、不裁决、不派活"),
            "档案禁区在 prompt 里"
        );
        assert!(
            prompt.contains("绕过用户确认直接改系统事实"),
            "§7 禁区原文在"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4：供给类失败 → 人话（剥前缀）；空店零炸（各路空/计数 0·照样能解释）。
    #[test]
    fn provider_failure_humanized_and_empty_stores_soft() {
        let dir = tmp_dir("failure");
        let state_path = dir.join("workflow-state.v0.json");
        fs::write(&state_path, "{}").expect("seed state file");
        // 空店（没 seed）→ facts 全空不炸。
        let facts = load_secretary_explain_facts(&state_path);
        assert!(facts.pending_proposals.is_empty());
        assert!(facts.supervisor_reminders.is_empty());
        assert_eq!(facts.pending_memory_candidate_count, 0);
        let failing = |_root: &str, _prompt: &str| {
            Err("codex_provider_unavailable:codex 额度用完了，明天再试".to_string())
        };
        let outcome = run_secretary_explain_core(&state_path, failing);
        assert_eq!(outcome.status, "unavailable");
        let reason = outcome.reason.unwrap();
        assert!(reason.contains("额度用完"), "人话直取：{reason}");
        assert!(!reason.contains("codex_provider_unavailable:"), "前缀剥掉");
        // 空回包 → unavailable 可重试。
        let empty = |_root: &str, _prompt: &str| Ok("   ".to_string());
        let outcome2 = run_secretary_explain_core(&state_path, empty);
        assert_eq!(outcome2.status, "unavailable");
        let _ = fs::remove_dir_all(dir);
    }
}
