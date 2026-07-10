// S3 项目主管 agent 第一刀：复用咨询 harness（只读 codex + tier-1 注入）。读已授权方案 + 项目上下文注入 →
// LM 只读拆解 → planned_tasks(目标/依赖/验收/汇报) → 喂 prepare_authorized_auto_dispatch.planned_tasks。
// 死线：主管只读(复用 readonly_codex_consult·项目不可写)、不碰闸/派发/确定性兜底、不自动派发；
// 每个 task 的 scope **取自已授权 scope_draft**(LM 不得扩范围)。本文件 include! 进 crate root（同 consultant_agent.rs）。

pub(crate) trait DirectorAgent {
    fn plan(
        &self,
        ctx: &ProjectContext,
        proposal: &ProjectConsultationProposal,
    ) -> Result<Vec<ProjectDirectorPlannedTask>, String>;

    // 2.1 批前预拆（待确认方案·仅预览）：与 plan 同拆解、只是 prompt 措辞不同（不对 LM 说「已授权」）。
    // 默认回落到 plan（stub 测试 director 无需另实现·返回同一份 canned 任务）。
    fn plan_preview(
        &self,
        ctx: &ProjectContext,
        proposal: &ProjectConsultationProposal,
    ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
        self.plan(ctx, proposal)
    }
}

const DIRECTOR_FINAL_REWORK_BUDGET: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DirectorFinalMarkDecision {
    Completed,
    NeedsRework,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirectorFinalMark {
    pub(crate) decision: DirectorFinalMarkDecision,
    pub(crate) reason: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DirectorFinalMarkContext {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) task_title: String,
    pub(crate) task_goal: String,
    pub(crate) acceptance_criteria: Vec<String>,
    pub(crate) report_status: Option<String>,
    pub(crate) acceptance_status: String,
    pub(crate) evidence_refs: Vec<String>,
    pub(crate) direction_risks: Vec<String>,
    pub(crate) yellow_reasons: Vec<String>,
    pub(crate) last_message_tail: String,
    pub(crate) rework_budget_remaining: usize,
}

pub(crate) trait DirectorFinalMarker {
    fn final_mark(&self, ctx: &DirectorFinalMarkContext) -> Result<DirectorFinalMark, String>;
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub(crate) struct DirectorWorkflowSummary {
    pub(crate) summary: String,
    pub(crate) key_facts: Vec<String>,
    pub(crate) open_items: Vec<String>,
    pub(crate) next_suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DirectorWorkflowSummaryContext {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) chain_run_id: String,
    pub(crate) total: usize,
    pub(crate) dispatched: usize,
    pub(crate) completed: usize,
    pub(crate) skipped: usize,
    pub(crate) steps: Vec<DirectorChainStep>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) trait DirectorSummaryGenerator {
    fn summarize_chain(
        &self,
        ctx: &DirectorWorkflowSummaryContext,
    ) -> Result<DirectorWorkflowSummary, String>;
}

#[cfg(test)]
struct FixturePassDirectorFinalMarker;

#[cfg(test)]
impl DirectorFinalMarker for FixturePassDirectorFinalMarker {
    fn final_mark(&self, _ctx: &DirectorFinalMarkContext) -> Result<DirectorFinalMark, String> {
        Ok(DirectorFinalMark {
            decision: DirectorFinalMarkDecision::Completed,
            reason: "测试夹具兼容：旧链测试不烧真实主管 LM。".to_string(),
        })
    }
}

#[cfg(test)]
struct FixtureDirectorSummaryGenerator;

#[cfg(test)]
impl DirectorSummaryGenerator for FixtureDirectorSummaryGenerator {
    fn summarize_chain(
        &self,
        ctx: &DirectorWorkflowSummaryContext,
    ) -> Result<DirectorWorkflowSummary, String> {
        Ok(DirectorWorkflowSummary {
            summary: format!("测试夹具主管总结：链 {} 已完成。", ctx.chain_run_id),
            key_facts: vec![format!("已完成任务数：{}", ctx.completed)],
            open_items: vec![],
            next_suggestions: vec!["测试夹具：候选仍需人工确认。".to_string()],
        })
    }
}

#[derive(Debug, Clone)]
struct DirectorFinalScreen {
    report_status: Option<String>,
    acceptance_status: String,
    evidence_refs: Vec<String>,
    direction_risks: Vec<String>,
    yellow_reasons: Vec<String>,
}

impl DirectorFinalScreen {
    fn is_green(&self) -> bool {
        self.yellow_reasons.is_empty()
    }
}

// ===== v0 静态主管档案 =====
const DIRECTOR_V0_PROFILE: &str = r#"你是「项目主管」。
职责:把已授权的方案拆成可派发给 worker 的具体任务。每个任务定清:做什么(task_goal)、依赖顺序(depends_on)、验收标准(acceptance_criteria)、汇报格式(report_format)。
铁律·自包含(最重要):**worker 在干净隔离上下文里执行,只看到这个任务的 task_goal 字符串——看不到这份方案、看不到别的任务、不能按需读文件。** 所以每个 task_goal 必须把执行所需的一切**写全进去**:目标文件的**完整路径**、**要写的具体内容**、依据的事实/数据**原样抄进来**。**绝不许写"按已注入方案/参见上文/见上一步/如方案所述"——worker 根本看不到那些。** 你已拿到方案,你的职责就是把它**翻译成 worker 只看 task_goal 就能独立干完的自包含指令**。
铁律·落地:只依据已注入的方案正文和项目上下文拆,不假设未注入的内容存在。任务对得上方案目标,不加方案没授权的事。
边界:只读、只规划、不执行、不自己派发。真派发由用户审过后走授权闸——你只产计划。
风格:任务粒度适中、依赖清晰、可验收;不堆废话。"#;

// 2.1：拆解 prompt 分「已授权」（auto_advance/合流·真派发前）与「待确认·仅预览」（批前预拆·别对 LM 说已授权）两措辞。
fn director_build_prompt(ctx: &ProjectContext, proposal: &ProjectConsultationProposal) -> String {
    director_build_prompt_variant(ctx, proposal, false)
}

fn director_build_prompt_variant(
    ctx: &ProjectContext,
    proposal: &ProjectConsultationProposal,
    is_preview: bool,
) -> String {
    let mut p = String::new();
    p.push_str(DIRECTOR_V0_PROFILE);
    p.push_str(if is_preview {
        "\n\n===== 待确认方案（仅预览·尚未授权·先看会拆成什么图）=====\n"
    } else {
        "\n\n===== 已授权方案（要拆的就是它）=====\n"
    });
    p.push_str(&format!(
        "方案标题: {}\n用户目标: {}\n一句话目标: {}\n",
        proposal.title, proposal.user_goal, proposal.goal_summary
    ));
    if !proposal.proposed_steps.is_empty() {
        p.push_str(&format!(
            "\n建议步骤:\n{}\n",
            proposal
                .proposed_steps
                .iter()
                .map(|s| format!("- {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    if !proposal.acceptance_criteria.is_empty() {
        p.push_str(&format!(
            "\n方案验收标准:\n{}\n",
            proposal.acceptance_criteria.join("\n")
        ));
    }
    p.push_str(&format!(
        "\n授权范围(scope·你拆的任务只能在此范围内,别扩): 读={:?} 写={:?} 工具={:?} 检查={:?} 必停={:?}\n",
        proposal.scope_draft.allowed_read_roots,
        proposal.scope_draft.allowed_write_roots,
        proposal.scope_draft.allowed_tools,
        proposal.scope_draft.allowed_checks,
        proposal.scope_draft.stop_conditions
    ));
    p.push_str("\n===== 项目上下文（已注入·只能依据这些）=====\n");
    if !ctx.document_map.is_empty() {
        p.push_str(&format!("文档地图:\n{}\n", ctx.document_map.join("\n")));
    }
    if !ctx.injected_documents.is_empty() {
        p.push_str("\n--- 项目文档正文（已注入·你读不到未注入文件）---\n");
        for (path, content) in &ctx.injected_documents {
            p.push_str(&format!("\n### 文件: {path}\n{content}\n"));
        }
    }
    // 刀B 补渲染（2026-07-08·质量债线报备逮到的既有暗债）：memory_summary 自刀B 起在重拆/预拆
    // 两处被填，但本渲染器一直没输出——填了没上脸=召回对主管半失效。补上（None 不渲染·与咨询侧
    // 「--- 项目记忆 ---」块同语义：参考不指令）。
    if let Some(memory) = &ctx.memory_summary {
        p.push_str(&format!(
            "\n--- 项目记忆（已确认的正式记忆·拆解时参考·仍以注入文档为准）---\n{memory}\n"
        ));
    }
    // 质量债·redo 幂等：重拆时喂「本单已完成事实」（只 re-plan 分支填此字段；首跑所批即所跑不经此
    // prompt、批前预拆不填 → 两处天然不渲染）。只事实不指令、只摘要不产物本体。
    if let Some(prior) = &ctx.prior_completed_summary {
        p.push_str(&format!(
            "\n--- 本单已完成（**别重复执行这些动作**·以下是已归档的 worker 自述）---\n{prior}\n重拆的新计划不得再包含与上面等价的动作；上轮超时/失败的任务可以拆细或简化后重排。\n"
        ));
    }
    p.push_str(
        r#"
===== 怎么拆 =====
把这份方案拆成有序的 worker 任务(通常 1-6 个)。只依据上面注入的方案+文档,不假设未注入内容。
**每个 task_goal 必须自包含**:把目标文件的完整路径、要写的具体内容、依据的事实**原样写进 task_goal**——worker 只看这段、不看方案/不看别的任务也能独立干完。**绝不写"参见方案/见上文/如上所述/见上一步"。**
report_format 写清 worker 该**结构化返回**什么(做了啥 / 产出在哪 / 成败),好让链/主管 parse 了往下走。
在最后输出且仅输出一个 ```json 代码块,是一个任务数组,严格这个结构:
[
  {
    "title": "任务名",
    "task_goal": "自包含完整指令:做什么 + 目标文件完整路径 + 要写的具体内容 + 依据(worker 只看这段就能干,不引用方案/别任务)",
    "target_role": "执行角色(如 codex-dev)",
    "depends_on": ["前置任务的 title(无前置则空数组)"],
    "acceptance_criteria": ["怎么算这个任务完成"],
    "report_format": ["worker 该结构化返回哪些:做了啥 / 产出在哪 / 成败"]
  }
]"#,
    );
    p
}

#[derive(serde::Deserialize)]
struct DirectorTaskJson {
    #[serde(default)]
    title: String,
    #[serde(default, alias = "objective")]
    task_goal: String,
    #[serde(default)]
    target_role: String,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    acceptance_criteria: Vec<String>,
    #[serde(default)]
    report_format: Vec<String>,
}

// 每个 task 的 scope 取自**已授权 scope_draft**（LM 不得扩范围）；target_role 用 LM 的（空则 codex-dev）。
fn director_task_scope_from_proposal(
    proposal: &ProjectConsultationProposal,
    target_role: &str,
) -> ProjectDirectorTaskScope {
    ProjectDirectorTaskScope {
        project_id: proposal.project_id.clone(),
        workflow_id: proposal.workflow_id.clone(),
        target_role: if target_role.trim().is_empty() {
            "codex-dev".to_string()
        } else {
            target_role.to_string()
        },
        task_package_kind: proposal
            .scope_draft
            .allowed_task_package_kinds
            .first()
            .cloned()
            .unwrap_or_else(|| "task_package".to_string()),
        allowed_read_scope: proposal.scope_draft.allowed_read_roots.clone(),
        allowed_write_scope: proposal.scope_draft.allowed_write_roots.clone(),
        callable_tool_capabilities: proposal.scope_draft.allowed_tools.clone(),
        required_checks: proposal.scope_draft.allowed_checks.clone(),
        stop_conditions: proposal.scope_draft.stop_conditions.clone(),
        timeout_policy: None,
        failure_policy: None,
        available_skills: vec![],
        available_knowledge_refs: vec![],
        forbidden_actions: vec![],
        model_id: None,
    }
}

// 从 codex 输出抠 json 任务数组 → planned_tasks。下游字段(work_item_id/guard_result/...)留空·由 prepare/派发机器填。
pub(crate) fn parse_director_plan(
    raw: &str,
    proposal: &ProjectConsultationProposal,
) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
    let json = consultant_extract_json_block(raw)
        .ok_or_else(|| "主管输出里没找到结构化 json 块".to_string())?;
    let tasks: Vec<DirectorTaskJson> =
        serde_json::from_str(&json).map_err(|error| format!("主管 plan json 解析失败:{error}"))?;
    if tasks.is_empty() {
        return Err("主管产出空任务列表".to_string());
    }
    let planned = tasks
        .into_iter()
        .enumerate()
        .map(|(index, task)| {
            let scope = director_task_scope_from_proposal(proposal, &task.target_role);
            let acceptance_criteria = if task.acceptance_criteria.is_empty() {
                proposal.acceptance_criteria.clone()
            } else {
                task.acceptance_criteria
            };
            ProjectDirectorPlannedTask {
                planned_task_id: format!("planned-task:{}:{}", proposal.workflow_id, index + 1),
                title: task.title,
                task_goal: task.task_goal,
                scope,
                depends_on: task.depends_on,
                acceptance_criteria,
                report_format: task.report_format,
                status: "planned".to_string(),
                // 下游字段留空——由 prepare_authorized_auto_dispatch / 派发机器填，主管不碰。
                guard_result: None,
                work_item_id: None,
                workflow_node_id: None,
                task_package_id: None,
                memory_packet_snapshot_id: None,
                prepared_dispatch_id: None,
                blocked_reasons: vec![],
            }
        })
        .collect();
    Ok(planned)
}

// ===== CliDirectorAgent（tier-1 impl：复用咨询的只读 codex）=====
pub(crate) struct CliDirectorAgent {
    pub(crate) timeout_ms: Option<i64>,
}

impl Default for CliDirectorAgent {
    fn default() -> Self {
        Self {
            timeout_ms: Some(420_000),
        }
    }
}

impl DirectorAgent for CliDirectorAgent {
    fn plan(
        &self,
        ctx: &ProjectContext,
        proposal: &ProjectConsultationProposal,
    ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
        let prompt = director_build_prompt(ctx, proposal);
        // 复用咨询的只读 confinement：readonly_codex_consult（read-only 沙箱·写盘根空·项目只读·不走执行闸）。
        let raw = codex_local_runner::readonly_codex_consult(
            &ctx.project_root,
            &prompt,
            self.timeout_ms,
        )?;
        parse_director_plan(&raw, proposal)
    }

    // 2.1 预拆：同一只读 confinement，只把 prompt 换成「待确认·仅预览」措辞（别对 LM 说已授权）。
    fn plan_preview(
        &self,
        ctx: &ProjectContext,
        proposal: &ProjectConsultationProposal,
    ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
        let prompt = director_build_prompt_variant(ctx, proposal, true);
        let raw = codex_local_runner::readonly_codex_consult(
            &ctx.project_root,
            &prompt,
            self.timeout_ms,
        )?;
        parse_director_plan(&raw, proposal)
    }
}

impl DirectorFinalMarker for CliDirectorAgent {
    fn final_mark(&self, ctx: &DirectorFinalMarkContext) -> Result<DirectorFinalMark, String> {
        let prompt = director_final_mark_prompt(ctx);
        let raw = codex_local_runner::readonly_codex_consult(
            &ctx.project_root,
            &prompt,
            self.timeout_ms,
        )?;
        parse_director_final_mark(&raw)
    }
}

impl DirectorSummaryGenerator for CliDirectorAgent {
    fn summarize_chain(
        &self,
        ctx: &DirectorWorkflowSummaryContext,
    ) -> Result<DirectorWorkflowSummary, String> {
        let prompt = director_workflow_summary_prompt(ctx);
        let raw = codex_local_runner::readonly_codex_consult(
            &ctx.project_root,
            &prompt,
            self.timeout_ms,
        )?;
        parse_director_workflow_summary(&raw)
    }
}

#[derive(serde::Deserialize)]
struct DirectorFinalMarkJson {
    #[serde(default)]
    decision: String,
    #[serde(default)]
    reason: String,
}

#[derive(serde::Deserialize)]
struct DirectorWorkflowSummaryJson {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    key_facts: Vec<String>,
    #[serde(default)]
    open_items: Vec<String>,
    #[serde(default)]
    next_suggestions: Vec<String>,
}

fn director_final_mark_prompt(ctx: &DirectorFinalMarkContext) -> String {
    format!(
        r#"你是项目主管，正在做每任务最终完成标记。你只判断这个 worker 任务是否可终标完成，或是否必须退回重做。

硬约束：
- 只能输出一个 ```json 代码块。
- decision 只能是 "completed" 或 "needs_rework"。
- 证据不足、验收不清、风险未处理时选 "needs_rework"。
- 不要要求用户补充，除非确实无法判断；这里需要的是任务级过/退回。

任务：
workflow_id：{workflow_id}
标题：{title}
目标：
{goal}

验收标准：
{criteria}

worker 回执摘要：
- report_status: {report_status}
- acceptance_status: {acceptance_status}
- evidence_refs: {evidence_refs}
- direction_risks: {direction_risks}
- deterministic_yellow_reasons: {yellow_reasons}
- rework_budget_remaining: {budget}

worker 最后消息尾部：
{tail}

输出格式：
```json
{{"decision":"completed|needs_rework","reason":"一句话说明主管判定理由"}}
```"#,
        workflow_id = ctx.workflow_id,
        title = ctx.task_title,
        goal = ctx.task_goal,
        criteria = if ctx.acceptance_criteria.is_empty() {
            "（未登记）".to_string()
        } else {
            ctx.acceptance_criteria.join("\n")
        },
        report_status = ctx
            .report_status
            .clone()
            .unwrap_or_else(|| "missing".to_string()),
        acceptance_status = ctx.acceptance_status,
        evidence_refs = if ctx.evidence_refs.is_empty() {
            "[]".to_string()
        } else {
            ctx.evidence_refs.join("；")
        },
        direction_risks = if ctx.direction_risks.is_empty() {
            "[]".to_string()
        } else {
            ctx.direction_risks.join("；")
        },
        yellow_reasons = ctx.yellow_reasons.join("；"),
        budget = ctx.rework_budget_remaining,
        tail = ctx.last_message_tail,
    )
}

fn parse_director_final_mark(raw: &str) -> Result<DirectorFinalMark, String> {
    let json = consultant_extract_json_block(raw)
        .ok_or_else(|| "主管终标输出里没找到结构化 json 块".to_string())?;
    let parsed: DirectorFinalMarkJson =
        serde_json::from_str(&json).map_err(|error| format!("主管终标 json 解析失败:{error}"))?;
    let decision = match parsed.decision.trim().to_lowercase().as_str() {
        "completed" | "complete" | "pass" | "passed" | "accepted" => {
            DirectorFinalMarkDecision::Completed
        }
        "needs_rework" | "needs-rework" | "rework" | "returned" | "return" | "needs_changes"
        | "needs-changes" => DirectorFinalMarkDecision::NeedsRework,
        other => return Err(format!("未知主管终标 decision：{other}")),
    };
    let reason = if parsed.reason.trim().is_empty() {
        "主管终标未给出理由".to_string()
    } else {
        parsed.reason.trim().to_string()
    };
    Ok(DirectorFinalMark { decision, reason })
}

fn director_workflow_summary_prompt(ctx: &DirectorWorkflowSummaryContext) -> String {
    let steps = ctx
        .steps
        .iter()
        .map(|step| {
            format!(
                "- {} [{}]: summary={} warning={} status={}",
                step.title,
                step.state,
                step.report_summary
                    .as_deref()
                    .unwrap_or("无 worker 摘要"),
                step.report_warning
                    .as_deref()
                    .unwrap_or("无 warning"),
                step.report_status.as_deref().unwrap_or("unknown")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let warnings = if ctx.warnings.is_empty() {
        "[]".to_string()
    } else {
        ctx.warnings.join("；")
    };
    format!(
        r#"你是项目主管，正在为一条已经 completed 的 worker 链写终局总结。总结只用于货脸呈现和记忆候选，不是完成闸。

硬约束：
- 只能输出一个 ```json 代码块。
- 不要包含 full transcript/raw stdout/raw stderr/prompt body/auth token/oauth/keychain/.env/rollout/provider credential 等敏感词。
- summary 用一句人话说明本链做成了什么。
- key_facts/open_items/next_suggestions 都是字符串数组；没有就给空数组。

链：
workflow_id: {workflow_id}
chain_run_id: {chain_run_id}
计数：total={total}, dispatched={dispatched}, completed={completed}, skipped={skipped}
warnings: {warnings}

任务结果：
{steps}

输出格式：
```json
{{"summary":"一句话工作流总结","key_facts":["关键事实"],"open_items":[],"next_suggestions":["后续建议"]}}
```"#,
        workflow_id = ctx.workflow_id,
        chain_run_id = ctx.chain_run_id,
        total = ctx.total,
        dispatched = ctx.dispatched,
        completed = ctx.completed,
        skipped = ctx.skipped,
        warnings = warnings,
        steps = steps,
    )
}

fn parse_director_workflow_summary(raw: &str) -> Result<DirectorWorkflowSummary, String> {
    let json = consultant_extract_json_block(raw)
        .ok_or_else(|| "主管总结输出里没找到结构化 json 块".to_string())?;
    let parsed: DirectorWorkflowSummaryJson =
        serde_json::from_str(&json).map_err(|error| format!("主管总结 json 解析失败:{error}"))?;
    let summary = parsed.summary.trim().to_string();
    if summary.is_empty() {
        return Err("主管总结 summary 为空".to_string());
    }
    Ok(DirectorWorkflowSummary {
        summary,
        key_facts: parsed
            .key_facts
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        open_items: parsed
            .open_items
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        next_suggestions: parsed
            .next_suggestions
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
    })
}

fn director_summary_body(summary: &DirectorWorkflowSummary) -> String {
    let mut lines = vec![summary.summary.clone()];
    if !summary.key_facts.is_empty() {
        lines.push(format!("关键事实：{}", summary.key_facts.join("；")));
    }
    if !summary.open_items.is_empty() {
        lines.push(format!("未决项：{}", summary.open_items.join("；")));
    }
    if !summary.next_suggestions.is_empty() {
        lines.push(format!("后续建议：{}", summary.next_suggestions.join("；")));
    }
    lines.join("\n")
}

fn director_summary_scope(project_root: &str, workflow_id: &str, timestamp: &str) -> MemoryScope {
    let pid = project_id(project_root);
    MemoryScope {
        scope_id: format!("memory-scope:workflow-summary:{}", stable_id(workflow_id)),
        scope_type: "workflow".to_string(),
        user_id: None,
        project_id: Some(pid),
        workflow_id: Some(workflow_id.to_string()),
        session_id: None,
        role_ids: vec!["project_director".to_string()],
        document_refs: vec![],
        permission_policy_ref: None,
        model_export_policy: "local_only".to_string(),
        valid_from: timestamp.to_string(),
        valid_until: None,
    }
}

fn capture_director_summary_candidate(
    path: &std::path::Path,
    project_root: &str,
    workflow_id: &str,
    chain_run_id: &str,
    summary: &DirectorWorkflowSummary,
    timestamp: &str,
) -> Result<CaptureMemoryEventOutput, String> {
    let pid = project_id(project_root);
    let source_ref_id = format!(
        "source:director-summary:{}:{}",
        stable_id(workflow_id),
        stable_id(chain_run_id)
    );
    let source_id = format!("director-summary:{chain_run_id}");
    let body = director_summary_body(summary);
    let input = CaptureMemoryEventInput {
        project_root: project_root.to_string(),
        project_id: Some(pid.clone()),
        workflow_id: Some(workflow_id.to_string()),
        workflow_node_id: None,
        run_unit_id: Some(chain_run_id.to_string()),
        product_command_id: None,
        product_attempt_id: None,
        runtime_log_ref: None,
        audit_refs: vec![format!("workflow_chain_director_summary:{chain_run_id}")],
        readback_ref: None,
        task_package_ref: None,
        memory_packet_ref: None,
        scope: director_summary_scope(project_root, workflow_id, timestamp),
        source_type: "final_review".to_string(),
        source_refs: vec![MemoryCaptureSourceRef {
            source_ref_id,
            source_type: "final_review".to_string(),
            source_id,
            project_id: Some(pid),
            workflow_id: Some(workflow_id.to_string()),
            workflow_node_id: None,
            run_unit_id: Some(chain_run_id.to_string()),
            product_command_id: None,
            product_attempt_id: None,
            runtime_log_ref: None,
            audit_ref_id: Some(format!("workflow_chain_director_summary:{chain_run_id}")),
            readback_ref: None,
            task_package_ref: None,
            memory_packet_ref: None,
            evidence_ref: Some(format!("workflow_chain_run:{chain_run_id}")),
            summary: summary.summary.clone(),
            sensitive_level: "internal".to_string(),
            created_at: timestamp.to_string(),
        }],
        summary: summary.summary.clone(),
        evidence_summary: format!("主管总结来自 completed 链运行 {chain_run_id}。"),
        sensitivity: "internal".to_string(),
        candidate_policy: "candidate_allowed".to_string(),
        generated_by_role: "project_director".to_string(),
        actor_id: "project-director:c4b".to_string(),
        risk_level: "low".to_string(),
        reason: "C4b 链末主管总结生成记忆候选；候选仍需确认门转正。".to_string(),
        candidate: Some(MemoryCaptureCandidateDraft {
            memory_type: "workflow_summary".to_string(),
            claim: summary.summary.clone(),
            body,
            review_reason: "C4b 从链末主管总结生成候选；这不是正式记忆，需人工确认。"
                .to_string(),
            requires_user_confirmation: true,
            actor_role: "project_director".to_string(),
        }),
        expected_capture_store_revision: None,
        expected_observation_store_revision: None,
        expected_candidate_store_revision: None,
    };
    memory_capture_bus::capture_event(
        path,
        &input,
        timestamp,
        &format!("director-summary-capture:{chain_run_id}"),
        &format!("director-summary-observation:{chain_run_id}"),
        &format!("director-summary-candidate:{chain_run_id}"),
    )
}

fn director_final_screen(
    task: &ProjectDirectorPlannedTask,
    report: Option<&worker_report::WorkerReport>,
) -> DirectorFinalScreen {
    let mut yellow_reasons = Vec::new();
    let Some(report) = report else {
        return DirectorFinalScreen {
            report_status: None,
            acceptance_status: "reported_not_completed".to_string(),
            evidence_refs: vec![],
            direction_risks: vec![],
            yellow_reasons: vec!["worker_report_missing".to_string()],
        };
    };

    let status = report.status.trim().to_string();
    let report_status = if status.is_empty() {
        None
    } else {
        Some(status.clone())
    };
    let has_help_signal = status.eq_ignore_ascii_case("blocked")
        || !report.permission_requests.is_empty()
        || !report.open_issues.is_empty()
        || !report.direction_risks.is_empty()
        || !report.follow_up_suggestions.is_empty();
    let acceptance_status = if has_help_signal {
        "blocked"
    } else {
        match status.to_lowercase().as_str() {
            "done" => "reported_completed",
            "partial" => "needs_rework",
            "failed" => "reported_not_completed",
            _ => "reported_not_completed",
        }
    }
    .to_string();

    if status != "done" {
        yellow_reasons.push(format!(
            "report_status_not_done:{}",
            if status.is_empty() { "missing" } else { &status }
        ));
    }
    if acceptance_status != "reported_completed" {
        yellow_reasons.push(format!("acceptance_status_not_completed:{acceptance_status}"));
    }
    if report.evidence.is_empty() {
        yellow_reasons.push("evidence_missing".to_string());
    }
    if !task.scope.required_checks.is_empty() {
        yellow_reasons.push(format!(
            "required_checks_unverified:{}",
            task.scope.required_checks.join(",")
        ));
    }
    if !report.direction_risks.is_empty() {
        yellow_reasons.push("direction_risks_present".to_string());
    }

    DirectorFinalScreen {
        report_status,
        acceptance_status,
        evidence_refs: report.evidence.clone(),
        direction_risks: report.direction_risks.clone(),
        yellow_reasons,
    }
}

fn tail_chars(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.trim().chars().collect();
    let start = chars.len().saturating_sub(n);
    chars[start..].iter().collect()
}

// ===== S3 主管→worker 链驱动（薄·按 depends_on 拓扑序跑 prepared 的 planned_tasks）=====
// 复用 execute_project_workflow_node_at（**S1 闸/沙箱·每节点 path-lock**）+ workflow_chain_topological_order
// （现成拓扑）+ chain controller 的链记录/停链/审计 helper（ensure_chain_run_record / set_chain_node_state /
// chain_run_stop_requested / finalize_chain_run / append_chain_audit）——全在 crate root 同模块（include!），
// 只调不改 → `workflow_chain_controller.rs` **本体 byte-0-diff**，无需开可见性。
// 4 护栏全在：① runaway 上限（max_nodes=min(max_tasks,任务数,硬顶50)）② **可中断**（每任务边界 read-fresh
// 查 stop_requested → 停；现成 `stop_project_workflow_chain` 命令按 workflow_id+running 能找到本驱动的链记录）
// ③ 审计（链起/每任务 start·done·skip·fail/链停·完成·失败 都进 audit_events）④ 可回滚（起链前 backup +
// execute 每派发 backup）。同-role 多任务共享 1 节点没关系——每次 execute 用**该任务自己的 work_item**
// （task_goal 各异）按序真跑；链记录的「节点」按 **planned_task_id** 编址（≠工作流 node_id，避免同-role 撞键）。
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct DirectorChainStep {
    pub(crate) planned_task_id: String,
    pub(crate) title: String,
    // "completed" | "failed" | "skipped"
    pub(crate) state: String,
    // fix·worker 回程契约：did（status）一句话摘要；无契约报文/非完成时 None（serde 加法·前端渐进接）。
    pub(crate) report_summary: Option<String>,
    // fix·worker 回程契约：每任务级报文诊断（落库失败 / 有输出没按契约）；无则 None。**不进链级 warnings**。
    pub(crate) report_warning: Option<String>,
    // 刀A·口供上脸：worker 自报 status（done|partial|failed）；没交口供 → None。前端据此判黄牌（呈现不驱动·黄牌不是闸）。
    pub(crate) report_status: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DirectorChainOutcome {
    pub(crate) total: usize,
    pub(crate) dispatched: usize,
    pub(crate) completed: usize,
    pub(crate) skipped: usize,
    pub(crate) chain_run_id: String,
    pub(crate) steps: Vec<DirectorChainStep>,
    pub(crate) director_summary: Option<DirectorWorkflowSummary>,
    pub(crate) warnings: Vec<String>,
    pub(crate) stopped_reason: Option<String>,
}

// 开工前绑定面板的逐任务选择。`existing` 必须在确认命令中绑到该任务自己的 node + work_item；
// `new` 仍由 C1 在派发前先生后绑，绝不把一个旧会话默认为整链共用。
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub(crate) struct ProjectDirectorTaskSessionBinding {
    pub(crate) planned_task_id: String,
    pub(crate) session_choice: String, // "new" | "existing"
    #[serde(default)]
    pub(crate) session_id: Option<String>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) struct ProjectDirectorFailedActionRequest {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) chain_run_id: String,
    pub(crate) planned_task_id: String,
    // "retry" | "rework" | "change_session" | "archive"
    pub(crate) action: String,
    pub(crate) actor_role: String,
    #[serde(default)]
    pub(crate) actor_id: Option<String>,
    #[serde(default)]
    pub(crate) explicit_retry_or_reopen: bool,
    #[serde(default)]
    pub(crate) planned_task: Option<ProjectDirectorPlannedTask>,
    #[serde(default)]
    pub(crate) max_nodes: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProjectDirectorFailedActionOutcome {
    pub(crate) action: String,
    pub(crate) chain_run_id: String,
    pub(crate) planned_task_id: String,
    pub(crate) transition_to: String,
    pub(crate) chain_state: String,
    pub(crate) node_state: String,
    pub(crate) new_session_id: Option<String>,
    pub(crate) chain_outcome: Option<DirectorChainOutcome>,
    pub(crate) warnings: Vec<String>,
    pub(crate) stopped_reason: Option<String>,
    pub(crate) message: String,
}

fn failed_action_task<'a>(
    request: &'a ProjectDirectorFailedActionRequest,
) -> Result<&'a ProjectDirectorPlannedTask, String> {
    let task = request
        .planned_task
        .as_ref()
        .ok_or_else(|| "四选一处置缺 planned_task，不能复用现有任务包/派发机器。".to_string())?;
    if task.planned_task_id != request.planned_task_id {
        return Err(format!(
            "四选一处置 planned_task_id 不一致：request={} task={}",
            request.planned_task_id, task.planned_task_id
        ));
    }
    if task.scope.workflow_id != request.workflow_id {
        return Err(format!(
            "四选一处置 workflow_id 不一致：request={} task={}",
            request.workflow_id, task.scope.workflow_id
        ));
    }
    Ok(task)
}

fn failed_action_task_node_id(task: &ProjectDirectorPlannedTask) -> String {
    format!(
        "{}:node:task:{}",
        task.scope.workflow_id,
        stable_id(&task.planned_task_id)
    )
}

fn failed_action_current_states(
    value: &Value,
    chain_run_id: &str,
    planned_task_id: &str,
) -> Result<(String, String), String> {
    let run = chain_run_record(value, chain_run_id)
        .ok_or_else(|| format!("找不到四选一处置目标链运行记录：{chain_run_id}"))?;
    let chain_state =
        optional_string_from(run, "state").unwrap_or_else(|| "unknown".to_string());
    let node_state = run
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| {
            nodes.iter().find(|node| {
                optional_string_from(node, "node_id").as_deref() == Some(planned_task_id)
            })
        })
        .and_then(|node| optional_string_from(node, "state"))
        .unwrap_or_else(|| "unknown".to_string());
    Ok((chain_state, node_state))
}

fn set_chain_run_state_for_failed_action(
    value: &mut Value,
    chain_run_id: &str,
    state: &str,
    timestamp: &str,
) -> Result<(), String> {
    let runs = value
        .get_mut("workflow_chain_runs")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "workflow_state 缺 workflow_chain_runs，无法处置四选一节点".to_string())?;
    let run = runs
        .iter_mut()
        .find(|run| optional_string_from(run, "chain_run_id").as_deref() == Some(chain_run_id))
        .ok_or_else(|| format!("找不到四选一处置目标链运行记录：{chain_run_id}"))?;
    run["state"] = json!(state);
    run["ended_at"] = if state == "running" {
        Value::Null
    } else {
        json!(timestamp)
    };
    run["stop_requested"] = json!(false);
    Ok(())
}

fn ensure_failed_node_transition(
    from: &str,
    to: &str,
    actor_role: &str,
    explicit_retry_or_reopen: bool,
) -> Result<(), String> {
    if workflow_node_transition_allowed(from, to, actor_role, explicit_retry_or_reopen) {
        Ok(())
    } else {
        Err(format!(
            "四选一节点处置被 transition_allowed 拒绝：{from}->{to} actor_role={actor_role}（需 project_director） explicit_retry_or_reopen={explicit_retry_or_reopen}"
        ))
    }
}

fn reopen_failed_chain_node_for_action(
    path: &std::path::Path,
    workflow_id: &str,
    chain_run_id: &str,
    task: &ProjectDirectorPlannedTask,
    from_state: &str,
    action: &str,
    message: &str,
) -> Result<(), String> {
    let ts = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    set_chain_run_state_for_failed_action(&mut value, chain_run_id, "running", &ts)?;
    set_chain_node_state(
        &mut value,
        chain_run_id,
        &task.planned_task_id,
        "running",
        None,
        Some(message),
    );
    update_node_state_for_id(&mut value, &failed_action_task_node_id(task), "running", &ts)?;
    append_chain_audit(
        &mut value,
        chain_run_id,
        workflow_id,
        &format!("workflow_chain_node_failed_action_{action}"),
        from_state,
        "running",
        &ts,
        message,
    )?;
    write_validated_workflow_state(path, &value)
}

#[allow(clippy::too_many_arguments)]
fn run_project_director_failed_action_inner(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    request: &ProjectDirectorFailedActionRequest,
    session_creator: Option<&dyn JiaobanNewSessionCreator>,
) -> Result<ProjectDirectorFailedActionOutcome, String> {
    let action = request.action.trim();
    let task = failed_action_task(request)?;
    let actor_id = request.actor_id.as_deref().unwrap_or("project_director");
    let value = read_workflow_state_value(path)?;
    let (chain_state, node_state) =
        failed_action_current_states(&value, &request.chain_run_id, &request.planned_task_id)?;
    if !matches!(node_state.as_str(), "failed" | "needs_rework") {
        return Err(format!(
            "四选一只能处置 failed / needs_rework 节点，当前节点状态是 {node_state}"
        ));
    }

    match action {
        "retry" | "change_session" => {
            if !workflow_transition_allowed(
                &chain_state,
                "running",
                request.explicit_retry_or_reopen,
            ) {
                return Err(format!(
                    "四选一链处置被 transition_allowed 拒绝：{chain_state}->running explicit_retry_or_reopen={}",
                    request.explicit_retry_or_reopen
                ));
            }
            ensure_failed_node_transition(
                &node_state,
                "running",
                &request.actor_role,
                request.explicit_retry_or_reopen,
            )?;
            require_test_project_path_lock(
                &request.project_root,
                "apply_project_director_failed_action",
            )?;
            require_active_authorization(path, &request.project_root, &request.workflow_id)?;
            let work_item_id = task
                .work_item_id
                .as_deref()
                .ok_or_else(|| "四选一重跑缺 work_item_id，无法复用现有派发机器。".to_string())?;
            let new_session_id = if action == "change_session" {
                let creator = session_creator.ok_or_else(|| {
                    "change_session 必须提供 C1 session_creator，不能绕过 create_and_bind_task_session。"
                        .to_string()
                })?;
                let node_id = task.workflow_node_id.as_deref().ok_or_else(|| {
                    "change_session 缺 workflow_node_id，无法复用 C1 create_and_bind。".to_string()
                })?;
                Some(create_and_bind_task_session(
                    path,
                    index,
                    &request.project_root,
                    &request.workflow_id,
                    node_id,
                    work_item_id,
                    task,
                    creator,
                )?)
            } else {
                None
            };
            if !reset_work_item_for_retry(path, &request.project_root, work_item_id) {
                return Err(format!(
                    "四选一 {action} 无法把 work_item {work_item_id} 复位到 ready_to_dispatch，已停手。"
                ));
            }
            reopen_failed_chain_node_for_action(
                path,
                &request.workflow_id,
                &request.chain_run_id,
                task,
                &node_state,
                action,
                if action == "change_session" {
                    "主管显式选择 change_session：已复用 C1 新建并绑定任务会话，交回现有链驱动重跑单任务。"
                } else {
                    "主管显式选择 retry：交回现有链驱动重跑单任务。"
                },
            )?;
            let one_task = vec![task.clone()];
            let chain_outcome = run_director_task_chain(
                path,
                index,
                readback_db_path,
                runner,
                &request.project_root,
                &request.workflow_id,
                &one_task,
                request.max_nodes.unwrap_or(1).max(1).min(1),
            )?;
            Ok(ProjectDirectorFailedActionOutcome {
                action: action.to_string(),
                chain_run_id: request.chain_run_id.clone(),
                planned_task_id: request.planned_task_id.clone(),
                transition_to: "running".to_string(),
                chain_state: "running".to_string(),
                node_state: "running".to_string(),
                new_session_id,
                chain_outcome: Some(chain_outcome),
                warnings: vec![],
                stopped_reason: None,
                message: format!(
                    "{node_state} 节点已由 {actor_id} 按 {action} 显式处置，并经现有链驱动单任务重跑。"
                ),
            })
        }
        "rework" => {
            ensure_failed_node_transition(&node_state, "needs_rework", &request.actor_role, false)?;
            let attempts_used = chain_node_usize_field(
                &value,
                &request.chain_run_id,
                &request.planned_task_id,
                "director_rework_attempts",
            );
            // 主管终标已把 needs_rework 的唯一预算用在「退回」本身；用户再次选 rework
            // 只是明确保留该待重做态，不重复扣预算、不自动重跑。
            let rework_already_requested = node_state == "needs_rework";
            if !rework_already_requested && attempts_used >= DIRECTOR_FINAL_REWORK_BUDGET {
                return Err(format!(
                    "director_rework_budget_exhausted:{}/{}，节点保持待主管选择其它处置。",
                    attempts_used, DIRECTOR_FINAL_REWORK_BUDGET
                ));
            }
            let work_item_id = task
                .work_item_id
                .as_deref()
                .ok_or_else(|| "四选一退回缺 work_item_id，无法复用 C4a reset。".to_string())?;
            let reset_ok = if rework_already_requested {
                true
            } else {
                reset_work_item_for_director_rework(path, &request.project_root, work_item_id)
            };
            let ts = unix_timestamp_string();
            let mut after = read_workflow_state_value(path)?;
            set_chain_node_state(
                &mut after,
                &request.chain_run_id,
                &request.planned_task_id,
                "needs_rework",
                None,
                Some("主管显式选择 rework：复用 C4a 退回预算和 reset。"),
            );
            set_chain_node_usize_field(
                &mut after,
                &request.chain_run_id,
                &request.planned_task_id,
                "director_rework_attempts",
                if rework_already_requested {
                    attempts_used
                } else {
                    attempts_used + 1
                },
            );
            set_chain_node_usize_field(
                &mut after,
                &request.chain_run_id,
                &request.planned_task_id,
                "director_rework_budget",
                DIRECTOR_FINAL_REWORK_BUDGET,
            );
            update_node_state_for_id(
                &mut after,
                &failed_action_task_node_id(task),
                "needs_rework",
                &ts,
            )?;
            append_chain_audit(
                &mut after,
                &request.chain_run_id,
                &request.workflow_id,
                "workflow_chain_node_failed_action_rework",
                &node_state,
                "needs_rework",
                &ts,
                if reset_ok {
                    "主管显式选择 rework：复用 C4a reset，work_item 已复位为可重做。"
                } else {
                    "主管显式选择 rework：复用 C4a reset，但 work_item 复位失败，待人工处理。"
                },
            )?;
            finalize_chain_run(&mut after, &request.chain_run_id, "stopped", &ts);
            append_chain_audit(
                &mut after,
                &request.chain_run_id,
                &request.workflow_id,
                "workflow_chain_run_stopped",
                &node_state,
                "stopped",
                &ts,
                &format!(
                    "{node_state} 节点退回 needs_rework，返工预算 {}/{}。",
                    if rework_already_requested {
                        attempts_used
                    } else {
                        attempts_used + 1
                    },
                    DIRECTOR_FINAL_REWORK_BUDGET
                ),
            )?;
            write_validated_workflow_state(path, &after)?;
            Ok(ProjectDirectorFailedActionOutcome {
                action: action.to_string(),
                chain_run_id: request.chain_run_id.clone(),
                planned_task_id: request.planned_task_id.clone(),
                transition_to: "needs_rework".to_string(),
                chain_state: "stopped".to_string(),
                node_state: "needs_rework".to_string(),
                new_session_id: None,
                chain_outcome: None,
                warnings: if reset_ok {
                    vec![]
                } else {
                    vec!["work_item_reset_failed".to_string()]
                },
                stopped_reason: Some("needs_rework:failed_action".to_string()),
                message: format!(
                    "{node_state} 节点已由 {actor_id} 按 rework 退回，复用 C4a 返工预算。"
                ),
            })
        }
        "archive" => {
            if !workflow_transition_allowed(&chain_state, "archived", false) {
                return Err(format!(
                    "四选一链结束被 transition_allowed 拒绝：{chain_state}->archived"
                ));
            }
            ensure_failed_node_transition(&node_state, "archived", &request.actor_role, false)?;
            let ts = unix_timestamp_string();
            let mut after = read_workflow_state_value(path)?;
            set_chain_node_state(
                &mut after,
                &request.chain_run_id,
                &request.planned_task_id,
                "archived",
                None,
                Some("主管显式选择 archive：按现成节点归档转移结束。"),
            );
            update_node_state_for_id(
                &mut after,
                &failed_action_task_node_id(task),
                "archived",
                &ts,
            )?;
            append_chain_audit(
                &mut after,
                &request.chain_run_id,
                &request.workflow_id,
                "workflow_chain_node_failed_action_archive",
                &node_state,
                "archived",
                &ts,
                "主管显式选择 archive：复用现成处置节点归档转移结束。",
            )?;
            finalize_chain_run(&mut after, &request.chain_run_id, "archived", &ts);
            write_validated_workflow_state(path, &after)?;
            Ok(ProjectDirectorFailedActionOutcome {
                action: action.to_string(),
                chain_run_id: request.chain_run_id.clone(),
                planned_task_id: request.planned_task_id.clone(),
                transition_to: "archived".to_string(),
                chain_state: "archived".to_string(),
                node_state: "archived".to_string(),
                new_session_id: None,
                chain_outcome: None,
                warnings: vec![],
                stopped_reason: Some("archived:failed_action".to_string()),
                message: format!("{node_state} 节点已由 {actor_id} 按 archive 结束。"),
            })
        }
        other => Err(format!(
            "未知四选一处置动作：{other}（允许 retry/rework/change_session/archive）"
        )),
    }
}

pub(crate) fn run_project_director_failed_action(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    request: &ProjectDirectorFailedActionRequest,
) -> Result<ProjectDirectorFailedActionOutcome, String> {
    run_project_director_failed_action_inner(path, index, readback_db_path, runner, request, None)
}

pub(crate) fn run_project_director_failed_action_with_session_creator(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    request: &ProjectDirectorFailedActionRequest,
    session_creator: &dyn JiaobanNewSessionCreator,
) -> Result<ProjectDirectorFailedActionOutcome, String> {
    run_project_director_failed_action_inner(
        path,
        index,
        readback_db_path,
        runner,
        request,
        Some(session_creator),
    )
}

// 2.4：重试前把 work_item 走**现成合法跳转**复位到 ready_to_dispatch（首次失败推离了它）——用现成
// update_work_item_state_at（限默认工作流），非默认工作流复位不成则返回 false（不重试·不硬闯状态机）。
// 逐步 fire（running→failed→needs_changes→ready_to_dispatch·非法跳转各步自忽略）。已在
// ready_to_dispatch 的终标退回任务直接成功，避免重复复位；其余以末步是否到位为准。
fn reset_work_item_for_retry(
    path: &std::path::Path,
    project_root: &str,
    work_item_id: &str,
) -> bool {
    let already_ready = read_workflow_state_value(path)
        .ok()
        .and_then(|value| {
            find_work_item(&value, &default_workflow_id(project_root), work_item_id)
                .and_then(|item| optional_string_from(item, "state"))
        })
        .as_deref()
        == Some("ready_to_dispatch");
    if already_ready {
        return true;
    }
    let step = |next_state: &str| {
        update_work_item_state_at(
            path,
            &WorkItemStateUpdateRequest {
                project_root: project_root.to_string(),
                work_item_id: work_item_id.to_string(),
                next_state: next_state.to_string(),
            },
        )
        .is_ok()
    };
    let _ = step("failed"); // 若卡在 running：running→failed（已 failed 则非法·忽略）
    let _ = step("needs_changes"); // failed/timed_out → needs_changes
    step("ready_to_dispatch") // needs_changes → ready_to_dispatch（末步·返回是否复位成功）
}

fn reset_work_item_for_director_rework(
    path: &std::path::Path,
    project_root: &str,
    work_item_id: &str,
) -> bool {
    let step = |next_state: &str| {
        update_work_item_state_at(
            path,
            &WorkItemStateUpdateRequest {
                project_root: project_root.to_string(),
                work_item_id: work_item_id.to_string(),
                next_state: next_state.to_string(),
            },
        )
        .is_ok()
    };
    let _ = step("needs_changes");
    step("ready_to_dispatch")
}

fn chain_node_usize_field(
    value: &Value,
    chain_run_id: &str,
    node_id: &str,
    field: &str,
) -> usize {
    chain_run_record(value, chain_run_id)
        .and_then(|run| run.get("nodes"))
        .and_then(Value::as_array)
        .and_then(|nodes| {
            nodes
                .iter()
                .find(|node| optional_string_from(node, "node_id").as_deref() == Some(node_id))
        })
        .and_then(|node| node.get(field))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

fn set_chain_node_usize_field(
    value: &mut Value,
    chain_run_id: &str,
    node_id: &str,
    field: &str,
    count: usize,
) {
    let Some(runs) = value
        .get_mut("workflow_chain_runs")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let Some(run) = runs
        .iter_mut()
        .find(|run| optional_string_from(run, "chain_run_id").as_deref() == Some(chain_run_id))
    else {
        return;
    };
    let Some(nodes) = run.get_mut("nodes").and_then(Value::as_array_mut) else {
        return;
    };
    if let Some(node) = nodes
        .iter_mut()
        .find(|node| optional_string_from(node, "node_id").as_deref() == Some(node_id))
    {
        node[field] = json!(count);
    }
}

// 2.4：判 tier-1 偶发早退（exit≠0 且非 timeout / 非沙箱-gate·记忆 real-codex-run-flaky）——这类才自动重试一次。
// 保守：Err（gate 拦 / spawn 失败等）不认作早退（不 retry）；state 已 completed 更不是。
fn is_tier1_early_exit(outcome: &Result<WorkflowNodeDispatchResult, String>) -> bool {
    match outcome {
        Ok(result) => {
            let dispatch = &result.dispatch;
            dispatch.state != "completed"
                && dispatch
                    .warnings
                    .iter()
                    .any(|w| w == "codex_resume_exit_nonzero")
                && !dispatch
                    .warnings
                    .iter()
                    .any(|w| w == "timeout" || w.contains("sandbox"))
                // fix8：供给类失败（额度/订阅/登录）不是抽风，重试=白等，排除不 retry。
                && !dispatch
                    .warnings
                    .iter()
                    .any(|w| w.contains("codex_provider_unavailable"))
        }
        Err(_) => false,
    }
}

// 2.4·拆步 retry：判 director.plan 是否 tier-1 偶发早退——codex 起了(过闸)但没落 last-message 文件。
// slice1 实测唯一确定信号 = `consult_last_message_read_failed`（记忆 jiaoban-retry-gap-director-consult-step）；
// 它只在 real_codex_executed=true 之后出现，与 gate 拦(readonly_blocked)/解析失败(json/空任务)/沙箱-gate/超时
// 各自的错误串互斥 → 这些**不命中**、不 retry（照 §2.4「gate/解析类不 retry」）。保守：只认这一个信号。
fn is_director_plan_flaky_early_exit(error: &str) -> bool {
    // fix8：供给类失败（额度/订阅/登录）不是抽风，重试=白等一分钟，明确排除。
    if error.contains("codex_provider_unavailable") {
        return false;
    }
    error.contains("consult_last_message_read_failed")
}

// 2.4：director.plan / plan_preview 拆步偶发早退 → **原地重试一次（不循环）**；二次失败照常报错。
// gate/解析类错误不命中判据 → 直接透传原错误（不 retry）。返回 (planned_tasks, 是否发生过重试)。
fn director_plan_with_retry(
    director: &dyn DirectorAgent,
    ctx: &ProjectContext,
    proposal: &ProjectConsultationProposal,
    preview: bool,
) -> Result<(Vec<ProjectDirectorPlannedTask>, bool), String> {
    let run = |director: &dyn DirectorAgent| {
        if preview {
            director.plan_preview(ctx, proposal)
        } else {
            director.plan(ctx, proposal)
        }
    };
    match run(director) {
        Ok(tasks) => Ok((tasks, false)),
        Err(error) if is_director_plan_flaky_early_exit(&error) => {
            // 偶发早退：原地重试一次（不循环）。
            run(director).map(|tasks| (tasks, true))
        }
        Err(error) => Err(error),
    }
}

// 2.2·所批即所跑：校验合流带进的「用户批过的图」——非空 + 每个任务 scope 的 project/workflow 必须与本授权一致
// （防串项目/串工作流的图混入）。**越界写范围等不在此判**——交下游 prepare guard 逐个钳/拒（只能拦不能扩·安全不降）。
fn validate_approved_planned_tasks(
    tasks: &[ProjectDirectorPlannedTask],
    workflow_id: &str,
    project_id_value: &str,
) -> Result<(), String> {
    if tasks.is_empty() {
        return Err("合流带入的「已批任务图」为空；拒绝（空图不跑）。".to_string());
    }
    for task in tasks {
        if task.scope.workflow_id != workflow_id {
            return Err(format!(
                "已批任务「{}」的 workflow_id（{}）与本次授权（{workflow_id}）不一致；拒绝。",
                task.title, task.scope.workflow_id
            ));
        }
        if task.scope.project_id != project_id_value {
            return Err(format!(
                "已批任务「{}」的 project_id（{}）与本次授权（{project_id_value}）不一致；拒绝。",
                task.title, task.scope.project_id
            ));
        }
    }
    Ok(())
}

// fix3 2.1·角色钳位（核心·治「莫名被拦」最大类）：主管 LM 常给任务自由编 target_role（如「reviewer」），
// 而授权 allowed_role_ids 只含档位那几个（codex-dev / project_director）→ 现状会撞 control_core guard 的
// 「目标角色不在授权范围内」→ blocked。这里把**界外角色归一到 codex-dev**（codex-dev 本就在授权名单·**只收不放**、
// 不改 write/tools/checks、绝不把界外名加进授权），并把一条人话警告带出（→ outcome/preview 的 warnings）。
// **只钳新拆产物**（plan / plan_preview 两路都过这里）；`validate_approved_planned_tasks` 不调本函数——回传数据
// 不静默改、界外照 guard 兜底拦（安全不降）。空 target_role 已在 director_task_scope_from_proposal 归 codex-dev。
fn clamp_planned_task_roles(
    tasks: &mut [ProjectDirectorPlannedTask],
    allowed_role_ids: &[String],
) -> Vec<String> {
    let mut warnings = Vec::new();
    for task in tasks.iter_mut() {
        if !allowed_role_ids.contains(&task.scope.target_role) {
            let original = task.scope.target_role.clone();
            task.scope.target_role = "codex-dev".to_string();
            warnings.push(format!(
                "任务「{}」的角色「{original}」不在授权名单，已按 codex-dev 执行。",
                task.title
            ));
        }
    }
    warnings
}

// fix4 2.1·残料接管（pre-prepare reconcile·核心）：上一轮跑挂会遗留**本轮同名**工作项（planned_task_id 是
// workflow+序号定址→重拆撞旧）——ready_for_review（活完成审查没记）/ running（派发后进程死）/ failed / timed_out。
// C4 保护（c4_c6:existing∈{running,ready_for_review,accepted,failed,timed_out,cancelled}→拒）**是对的、不改**；
// 这里在 prepare **之前**，把**本轮 planned ids 派生的**遗留工作项走**合法状态机**（复用 reset_work_item_for_retry·
// 每步经 update_work_item_state_at·合法迁移自带审计·**绝不直接改 JSON state 字段**）复位到 ready_to_dispatch、
// 离开被保护状态。**只扫本轮 ids**（canvas-run 等别人的残料不碰）；accepted/cancelled（终态/人工态）**不接管**、
// 留 C4 照拒（报错文案已可行动·fix3-UI 兜底）。合法迁移表已核（control_core：ready_for_review→needs_changes 等全在）。
fn reconcile_stale_work_items_for_plan(
    path: &std::path::Path,
    project_root: &str,
    planned_tasks: &[ProjectDirectorPlannedTask],
) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut reconciled = 0usize;
    // 只读一次快照扫各任务状态（work_item_id 各异·互不影响；复位走 update_work_item_state_at 各自 read-fresh）。
    let Ok(value) = read_workflow_state_value(path) else {
        return warnings; // 读不到状态就不接管（prepare 会照常处理/报错）。
    };
    for task in planned_tasks {
        let work_item_id = c4_work_item_id(&task.scope.workflow_id, &task.planned_task_id);
        let Some(state) = find_work_item(&value, &task.scope.workflow_id, &work_item_id)
            .and_then(|item| optional_string_from(item, "state"))
        else {
            continue; // 本轮该任务尚无工作项（干净）→ 无需接管。
        };
        match state.as_str() {
            // 遗留活料：走合法行走器复位到 ready_to_dispatch（离开 C4 被保护状态·重新可派）。
            "running" | "ready_for_review" | "failed" | "timed_out" => {
                if reset_work_item_for_retry(path, project_root, &work_item_id) {
                    reconciled += 1;
                } else {
                    warnings.push(format!(
                        "工作项「{work_item_id}」（{state}）合法复位未走通，交 C4 照拒兜底。"
                    ));
                }
            }
            // 终态/人工态：不接管（罕见）——留 C4 照拒。
            "accepted" | "cancelled" => {
                warnings.push(format!(
                    "工作项「{work_item_id}」处于 {state}（终态/人工态），不接管、交 C4 处理。"
                ));
            }
            // draft / ready_to_dispatch / needs_changes / paused 等——无需接管。
            _ => {}
        }
    }
    if reconciled > 0 {
        warnings.insert(
            0,
            format!("已接管上一轮遗留的 {reconciled} 个工作项（合法打回、重新派发）。"),
        );
    }
    warnings
}

// fix4 2.2·重拆即新链（fresh-run·**只 re-plan 路径调**）：起链前把该 workflow **未收尾**的旧链记录
// （state∈{running,stopped}·镜像 ensure_chain_run_record 的续跑判据）用现成 finalize_chain_run 正式标结为
// superseded（+ 审计「被新一轮重拆取代」），之后 ensure 走**新建记录**分支（不再跨轮乱续）。controller 本体
// 0-diff（只调 finalize_chain_run/append_chain_audit）。返回 Some(人话) 表示标结过、供 outcome warnings 用。
fn finalize_stale_chain_for_replan(
    path: &std::path::Path,
    project_root: &str,
    workflow_id: &str,
) -> Result<Option<String>, String> {
    let mut value = read_workflow_state_value(path)?;
    let pid = project_id(project_root);
    let stale = value
        .get("workflow_chain_runs")
        .and_then(Value::as_array)
        .and_then(|runs| {
            runs.iter()
                .find(|run| {
                    optional_string_from(run, "workflow_id").as_deref() == Some(workflow_id)
                        && optional_string_from(run, "project_id").as_deref() == Some(pid.as_str())
                        && matches!(
                            optional_string_from(run, "state").as_deref(),
                            Some("running") | Some("stopped")
                        )
                })
                .map(|run| {
                    (
                        optional_string_from(run, "chain_run_id").unwrap_or_default(),
                        optional_string_from(run, "state").unwrap_or_default(),
                    )
                })
        });
    let Some((chain_run_id, before_state)) = stale else {
        return Ok(None); // 没有未收尾旧链 → 无需标结（干净起链）。
    };
    let ts = unix_timestamp_string();
    finalize_chain_run(&mut value, &chain_run_id, "superseded", &ts);
    append_chain_audit(
        &mut value,
        &chain_run_id,
        workflow_id,
        "workflow_chain_run_superseded",
        &before_state,
        "superseded",
        &ts,
        "上一轮未收尾的链记录被新一轮重拆取代，已正式标结（本轮从头重跑）。",
    )?;
    write_validated_workflow_state(path, &value)?;
    Ok(Some(
        "上一轮未收尾的运行已标结，本轮从头重跑（不再接续）。".to_string(),
    ))
}

// ===== 质量债·redo 幂等：收集「本单已完成事实」（喂重拆·2026-07-06 双删案）=====
// 全走 audit_events 时间窗（授权 created_at 之后 → 多轮叠加 A+B 都在）：
//   ① 口供事件（worker_structured_report_recorded·B1 同一读法同构）→ 「任务标题」— did 首行（status）+ 产物文件名；
//   ② 完成但没交口供：链完成审计（workflow_chain_node_completed）里有、口供里没有的任务 → 「标题」—（无自述·执行态 completed）；
//   ③ 超时事实：链失败审计 reason 含 ·timed_out 标记（classify 现成 "timeout" 信号的忠实投影）→ 「标题」— 上轮超时被杀。
// 0 条 → None（现状）；读失败 → None 不挡重拆（增益不是闸·同记忆召回先例）。
// 词表死线：只事实摘要（did/status/产物**文件名**），不搬产物内容本体（「产物喂下一步」用户明令另批）。
fn collect_prior_completed_summary(
    path: &std::path::Path,
    workflow_id: &str,
    authorization_created_at_ms: i64,
) -> Option<String> {
    let value = read_workflow_state_value(path).ok()?;
    let events = value.get("audit_events")?.as_array()?;
    let in_window = |event: &&Value| -> bool {
        optional_string_from(event, "workflow_id").as_deref() == Some(workflow_id)
            && optional_string_from(event, "created_at")
                .and_then(|created| created.parse::<i64>().ok())
                .map(|created| created >= authorization_created_at_ms)
                .unwrap_or(false)
    };
    let clip = |text: &str, max: usize| -> String {
        let first_line = text.trim().lines().next().unwrap_or("").trim();
        let chars: Vec<char> = first_line.chars().collect();
        if chars.len() <= max {
            first_line.to_string()
        } else {
            format!("{}…", chars[..max].iter().collect::<String>())
        }
    };
    // 审计 reason 里抠「任务「X」」的标题。
    let title_from_reason = |reason: &str| -> Option<String> {
        let start = reason.find('「')? + '「'.len_utf8();
        let end = reason[start..].find('」')? + start;
        Some(reason[start..end].to_string())
    };
    let mut lines: Vec<String> = Vec::new();
    let mut reported_titles: Vec<String> = Vec::new();
    // ① 口供行（标题从 work_items 拿·prepare 用 task.title 写入）。
    for event in events.iter().filter(|event| {
        in_window(event)
            && optional_string_from(event, "event_type").as_deref()
                == Some("worker_structured_report_recorded")
    }) {
        let work_item_id = optional_string_from(event, "work_item_id").unwrap_or_default();
        let title = find_work_item(&value, workflow_id, &work_item_id)
            .and_then(|item| optional_string_from(item, "title"))
            .unwrap_or_else(|| "（未命名任务）".to_string());
        let did = clip(
            &optional_string_from(event, "executed_what").unwrap_or_default(),
            120,
        );
        let status = optional_string_from(event, "acceptance_status").unwrap_or_default();
        let outputs = clip(
            &optional_string_from(event, "changed_what").unwrap_or_default(),
            120,
        );
        lines.push(format!("「{title}」— {did}（{status}）；产物：{outputs}"));
        reported_titles.push(title);
    }
    // ② 完成但没交口供（链完成审计有、口供没有）。
    for event in events.iter().filter(|event| {
        in_window(event)
            && optional_string_from(event, "event_type").as_deref()
                == Some("workflow_chain_node_completed")
    }) {
        if let Some(title) =
            optional_string_from(event, "reason").and_then(|reason| title_from_reason(&reason))
        {
            if !reported_titles.contains(&title) {
                lines.push(format!("「{title}」—（无自述·执行态 completed）"));
                reported_titles.push(title);
            }
        }
    }
    // ③ 上轮超时事实（喂「考虑拆细」·[接着跑]人肉路径同样受益——读盘不依赖谁触发的重拆）。
    for event in events.iter().filter(|event| {
        in_window(event)
            && optional_string_from(event, "event_type").as_deref()
                == Some("workflow_chain_node_failed")
            && optional_string_from(event, "reason")
                .map(|reason| reason.contains("·timed_out"))
                .unwrap_or(false)
    }) {
        if let Some(title) =
            optional_string_from(event, "reason").and_then(|reason| title_from_reason(&reason))
        {
            let line = format!("「{title}」— 上轮超时被杀（考虑拆细或简化后重排）");
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
    }
    if lines.is_empty() {
        return None;
    }
    lines.truncate(12);
    Some(lines.join("\n"))
}

// ===== 质量债·超时反馈边：从链停因判「任务超时导致的 fail-stop」（不新造分类——
// `·timed_out` 标记来自 classify 落在 dispatch.warnings 的现成 "timeout" 信号，链 fail_msg 忠实带出：
// `fail_stop:node_error:{title}:worker 派发未完成（state=failed·timed_out）`）。
// 供给类（Err 路）/gate 拒（Err 路）/普通 failed（无标记）都不含 `·timed_out` → 不触发。
fn chain_timeout_fail_stop_task(stopped_reason: Option<&str>) -> Option<String> {
    let reason = stopped_reason?;
    let rest = reason.strip_prefix("fail_stop:node_error:")?;
    if !reason.contains("·timed_out") {
        return None;
    }
    Some(
        rest.rsplit_once(':')
            .map(|(title, _)| title)
            .unwrap_or(rest)
            .to_string(),
    )
}

fn worker_help_message(help: &worker_report::WorkerReportHelpSignal) -> String {
    let mut parts = vec![format!("worker 求助·待主管：{}", help.summary)];
    if !help.permission_requests.is_empty() {
        parts.push(format!("权限/资料：{}", help.permission_requests.join("；")));
    }
    if !help.open_issues.is_empty() {
        parts.push(format!("卡点：{}", help.open_issues.join("；")));
    }
    if !help.direction_risks.is_empty() {
        parts.push(format!("方向风险：{}", help.direction_risks.join("；")));
    }
    if !help.follow_up_suggestions.is_empty() {
        parts.push(format!("建议：{}", help.follow_up_suggestions.join("；")));
    }
    parts.join("；")
}

// 反馈边放行判定（抽出供单测直击）：停因是任务超时 && 用户没点过停（读盘上链记录 stop_requested·
// 现成 helper；读不到盘 → 保守不自动续）→ Some(超时任务标题)。
fn timeout_auto_replan_decision(
    path: &std::path::Path,
    chain: &DirectorChainOutcome,
) -> Option<String> {
    let title = chain_timeout_fail_stop_task(chain.stopped_reason.as_deref())?;
    let user_asked_stop = read_workflow_state_value(path)
        .map(|value| chain_run_stop_requested(&value, &chain.chain_run_id))
        .unwrap_or(true);
    if user_asked_stop {
        None
    } else {
        Some(title)
    }
}

// fix5·超龄判据（推导）：worker 单次派发 timeout=600s（commands.rs:2352·max_retries=0），runner 到时**必杀** →
// 一条 running 派发记录的**物理最长窗口 = 600s**。翻倍容错（刀1 链级 retry 会另起记录、时间戳滞后、时钟偏移）=
// 20 分钟；再加余量 → **30 分钟**。超过它还 running 的派发**物理上不可能仍活着**——唯一解释是进程中途死了、
// 留下永久墓碑（has_inflight_dispatch 只数 running → 卡死该工作流一切新派发）。> 物理上限 = 绝不误杀真活。
const STALE_RUNNING_DISPATCH_MS: i64 = 1_800_000; // 30 分钟

// fix5·残料终章（中断遗留的 running 派发记录标结）：挂在 fix4 reconcile 同位置（auto_advance 两分支合流后·
// prepare 前）。扫**本轮 (workflow, 角色节点)** 上的 running 派发记录：**确证已死**（age>物理上限）的标结为 failed
// （解除 S1 的 duplicate_blocked）+ 审计；**未超龄/缺时间戳**的**绝不碰**（可能真在跑·留给闸拦=保护原语义·出人话）。
// 派发记录**无迁移表**（状态由 execute 流内联写·commands 注释）——这是包 §2.2 批准的**受控定点接管**：只改匹配记录的
// **state + warnings 两字段**（别的字段/别的记录一律不动）、必带审计。S1 闸/has_inflight/execute 一字不动。
fn reconcile_stale_running_dispatches(
    path: &std::path::Path,
    planned_tasks: &[ProjectDirectorPlannedTask],
    now_ms: i64,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let Ok(mut value) = read_workflow_state_value(path) else {
        return warnings; // 读不到状态就不接管（prepare/闸会照常处理）。
    };
    // 本轮节点集：planned_tasks 的 target_role（fix3 钳位后）→ c4_node_id 去重。**只碰这些**。
    let node_ids: std::collections::BTreeSet<String> = planned_tasks
        .iter()
        .map(|task| c4_node_id(&task.scope.workflow_id, &task.scope.target_role))
        .collect();
    if node_ids.is_empty() {
        return warnings;
    }
    // 第一遍（只读）：找本轮节点上的 running 派发，判超龄。
    let mut to_supersede: Vec<(String, String)> = Vec::new(); // (dispatch_id, node_id)
    if let Some(dispatches) = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
    {
        for dispatch in dispatches {
            // 只碰 running（prepared 是每次派发的 orphan·闸不数·不碰；failed/completed/timed_out 终态不碰）。
            if optional_string_from(dispatch, "state").as_deref() != Some("running") {
                continue;
            }
            let Some(node_id) = optional_string_from(dispatch, "node_id") else {
                continue;
            };
            if !node_ids.contains(&node_id) {
                continue; // 别的 workflow / canvas / 别的节点——不扫。
            }
            let Some(dispatch_id) = optional_string_from(dispatch, "dispatch_id") else {
                continue;
            };
            // age = now - started_at_ms（缺则退到 created_at_ms）。两者都缺 → 无法证明超龄 → 不碰（防误杀）。
            match i64_value(dispatch, "started_at_ms")
                .or_else(|| i64_value(dispatch, "created_at_ms"))
            {
                Some(started_ms)
                    if now_ms.saturating_sub(started_ms) > STALE_RUNNING_DISPATCH_MS =>
                {
                    to_supersede.push((dispatch_id, node_id));
                }
                Some(started_ms) => {
                    let minutes = now_ms.saturating_sub(started_ms) / 60_000;
                    warnings.push(format!(
                        "节点「{node_id}」可能仍有一次执行没结束（约 {minutes} 分钟前起）；本轮若被拦请稍后再试。"
                    ));
                }
                None => {
                    warnings.push(format!(
                        "节点「{node_id}」有一条 running 执行记录但缺起始时间戳，无法判超龄、不接管（本轮若被拦请稍后再试）。"
                    ));
                }
            }
        }
    }
    if to_supersede.is_empty() {
        return warnings;
    }
    // 第二遍（定点接管）：只改匹配记录的 state→failed + append 一条 warnings 说明（别的字段不动）。
    if let Some(dispatches) = value
        .get_mut("workflow_node_dispatches")
        .and_then(Value::as_array_mut)
    {
        for dispatch in dispatches.iter_mut() {
            let Some(dispatch_id) = optional_string_from(dispatch, "dispatch_id") else {
                continue;
            };
            if !to_supersede.iter().any(|(id, _)| id == &dispatch_id) {
                continue;
            }
            dispatch["state"] = json!("failed");
            if let Some(list) = dispatch.get_mut("warnings").and_then(Value::as_array_mut) {
                list.push(json!(
                    "上次运行进程中断遗留，已由新一轮标结（stale_running_dispatch_superseded）。"
                ));
            }
        }
    }
    // 审计（每条一 event·带 dispatch_id/node_id·只 append）。
    let ts = unix_timestamp_string();
    for (dispatch_id, node_id) in &to_supersede {
        if let Ok(events) = array_mut(&mut value, "audit_events") {
            events.push(json!({
                "event_id": format!("stale-running-dispatch-superseded:{dispatch_id}:{ts}"),
                "event_type": "stale_running_dispatch_superseded",
                "target_ref": node_id,
                "actor_ref": "role_loop_auto_advance",
                "source_kind": "stale_dispatch_reconcile",
                "permission_level": "workflow_event_record",
                "dispatch_id": dispatch_id,
                "created_at": ts,
                "reason": "上次运行进程中断遗留的 running 派发记录（超龄·物理上不可能仍在跑），已标结 failed 以解除 duplicate_blocked。",
            }));
        }
    }
    if write_validated_workflow_state(path, &value).is_ok() {
        warnings.insert(
            0,
            format!(
                "已标结 {} 条上次中断遗留的执行记录（超龄·解除 duplicate_blocked）。",
                to_supersede.len()
            ),
        );
    }
    warnings
}

// C1·每任务独立会话：**公有签名不变**（lib.rs 0-diff·手动挡/旧测试照走）。无 session_creator = 拐杖退役后的
// 只读兼容路径（沿用节点旧绑定 resume）；C1 生产主路径走 run_director_task_chain_with_session_creator。
pub(crate) fn run_director_task_chain(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    project_root: &str,
    workflow_id: &str,
    tasks: &[ProjectDirectorPlannedTask],
    max_tasks: usize,
) -> Result<DirectorChainOutcome, String> {
    #[cfg(test)]
    let final_marker = FixturePassDirectorFinalMarker;
    #[cfg(not(test))]
    let final_marker = CliDirectorAgent::default();
    #[cfg(test)]
    let summary_generator = FixtureDirectorSummaryGenerator;
    #[cfg(not(test))]
    let summary_generator = CliDirectorAgent::default();
    run_director_task_chain_inner(
        path,
        index,
        readback_db_path,
        runner,
        project_root,
        workflow_id,
        tasks,
        max_tasks,
        None,
        None,
        &final_marker,
        &summary_generator,
    )
}

pub(crate) fn run_director_task_chain_with_final_marker(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    project_root: &str,
    workflow_id: &str,
    tasks: &[ProjectDirectorPlannedTask],
    max_tasks: usize,
    final_marker: &dyn DirectorFinalMarker,
) -> Result<DirectorChainOutcome, String> {
    #[cfg(test)]
    let summary_generator = FixtureDirectorSummaryGenerator;
    #[cfg(not(test))]
    let summary_generator = CliDirectorAgent::default();
    run_director_task_chain_with_markers(
        path,
        index,
        readback_db_path,
        runner,
        project_root,
        workflow_id,
        tasks,
        max_tasks,
        final_marker,
        &summary_generator,
    )
}

pub(crate) fn run_director_task_chain_with_markers(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    project_root: &str,
    workflow_id: &str,
    tasks: &[ProjectDirectorPlannedTask],
    max_tasks: usize,
    final_marker: &dyn DirectorFinalMarker,
    summary_generator: &dyn DirectorSummaryGenerator,
) -> Result<DirectorChainOutcome, String> {
    run_director_task_chain_inner(
        path,
        index,
        readback_db_path,
        runner,
        project_root,
        workflow_id,
        tasks,
        max_tasks,
        None,
        None,
        final_marker,
        summary_generator,
    )
}

// C1 主路径：每任务派发前经现成先生后绑建一条以任务命名的专属新会话 → 绑到本任务角色节点 → 该任务用新会话
// resume。worker 只吃工作台发的任务包（上下文隔离从「顺带记得」变「制度保证」的会话半边）。
pub(crate) fn run_director_task_chain_with_session_creator(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    project_root: &str,
    workflow_id: &str,
    tasks: &[ProjectDirectorPlannedTask],
    max_tasks: usize,
    session_creator: &dyn JiaobanNewSessionCreator,
) -> Result<DirectorChainOutcome, String> {
    #[cfg(test)]
    let final_marker = FixturePassDirectorFinalMarker;
    #[cfg(not(test))]
    let final_marker = CliDirectorAgent::default();
    #[cfg(test)]
    let summary_generator = FixtureDirectorSummaryGenerator;
    #[cfg(not(test))]
    let summary_generator = CliDirectorAgent::default();
    run_director_task_chain_with_session_creator_and_final_marker(
        path,
        index,
        readback_db_path,
        runner,
        project_root,
        workflow_id,
        tasks,
        max_tasks,
        session_creator,
        &final_marker,
        &summary_generator,
    )
}

// 开工前绑定面板确认后的混合路径：已有会话已在 prepare 后逐任务绑好，只有标为 new 的任务进入 C1。
// C1 本体/runner/执行机不改；映射缺项在链内再次拒绝，绝不静默回落到共用旧会话。
fn run_director_task_chain_with_task_session_bindings(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    project_root: &str,
    workflow_id: &str,
    tasks: &[ProjectDirectorPlannedTask],
    max_tasks: usize,
    session_creator: &dyn JiaobanNewSessionCreator,
    task_session_bindings: &std::collections::BTreeMap<String, ProjectDirectorTaskSessionBinding>,
) -> Result<DirectorChainOutcome, String> {
    #[cfg(test)]
    let final_marker = FixturePassDirectorFinalMarker;
    #[cfg(not(test))]
    let final_marker = CliDirectorAgent::default();
    #[cfg(test)]
    let summary_generator = FixtureDirectorSummaryGenerator;
    #[cfg(not(test))]
    let summary_generator = CliDirectorAgent::default();
    run_director_task_chain_inner(
        path,
        index,
        readback_db_path,
        runner,
        project_root,
        workflow_id,
        tasks,
        max_tasks,
        Some(session_creator),
        Some(task_session_bindings),
        &final_marker,
        &summary_generator,
    )
}

pub(crate) fn run_director_task_chain_with_session_creator_and_final_marker(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    project_root: &str,
    workflow_id: &str,
    tasks: &[ProjectDirectorPlannedTask],
    max_tasks: usize,
    session_creator: &dyn JiaobanNewSessionCreator,
    final_marker: &dyn DirectorFinalMarker,
    summary_generator: &dyn DirectorSummaryGenerator,
) -> Result<DirectorChainOutcome, String> {
    run_director_task_chain_inner(
        path,
        index,
        readback_db_path,
        runner,
        project_root,
        workflow_id,
        tasks,
        max_tasks,
        Some(session_creator),
        None,
        final_marker,
        summary_generator,
    )
}

fn run_director_task_chain_inner(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    project_root: &str,
    workflow_id: &str,
    tasks: &[ProjectDirectorPlannedTask],
    max_tasks: usize,
    session_creator: Option<&dyn JiaobanNewSessionCreator>,
    task_session_bindings: Option<&std::collections::BTreeMap<String, ProjectDirectorTaskSessionBinding>>,
    final_marker: &dyn DirectorFinalMarker,
    summary_generator: &dyn DirectorSummaryGenerator,
) -> Result<DirectorChainOutcome, String> {
    use std::collections::BTreeSet;

    // 死线·圈固定测试项目（高危#4-轻档前提）：非测试 root 在驱动入口直接拒——连链记录都不建、零副作用。
    // 与现成命令同款 path-lock（require_test_project_path_lock·纯路径检查）；闸/沙箱本体不动。execute 每节点
    // 另有 S1 闸，这里是入口侧第二道（defense-in-depth），坐实「圈测试项目」、防非测试 root 留下脏链记录。
    require_test_project_path_lock(project_root, "run_director_task_chain")?;

    // F5·健壮：重复 title 会让拓扑/find 取第一个、后一个永不跑 → 直接拒（不静默丢任务）。
    let mut seen_titles: BTreeSet<&str> = BTreeSet::new();
    for task in tasks {
        if !seen_titles.insert(task.title.as_str()) {
            return Err(format!(
                "planned_tasks 含重复 title「{}」——拓扑按 title 建边会让后一个任务永不跑，拒绝起链",
                task.title
            ));
        }
    }
    let titles: Vec<String> = tasks.iter().map(|task| task.title.clone()).collect();
    let title_set: BTreeSet<&str> = titles.iter().map(String::as_str).collect();

    // 拓扑边 = depends_on；F5·健壮：依赖指向不存在的 title → 记 warning（不静默丢，便于排错），不建该边。
    let mut warnings: Vec<String> = Vec::new();
    let mut edges: Vec<(String, String)> = Vec::new();
    for task in tasks {
        for dep in &task.depends_on {
            if title_set.contains(dep.as_str()) {
                edges.push((dep.clone(), task.title.clone()));
            } else {
                warnings.push(format!(
                    "任务「{}」依赖不存在的前置「{dep}」——该依赖被忽略（拓扑序不含它）",
                    task.title
                ));
            }
        }
    }
    let order_titles = workflow_chain_topological_order(&titles, &edges)?;
    // 链记录的节点序 = 拓扑序的 planned_task_id（每个任务一个节点，按 planned_task_id 编址）。
    let order_task_ids: Vec<String> = order_titles
        .iter()
        .filter_map(|title| tasks.iter().find(|task| &task.title == title))
        .map(|task| task.planned_task_id.clone())
        .collect();

    let total = tasks.len();
    // runaway 上限：min(请求, 任务数, 硬顶 50)，至少 1（同 controller 语义）。
    let max_nodes = max_tasks
        .min(total)
        .min(WORKFLOW_CHAIN_MAX_NODES_HARD_CAP)
        .max(1);
    let pid = project_id(project_root);

    // 起链：建/续 running 链记录 + 起链前 backup（可回滚）+ 审计「链起」。
    let start_ts = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    // fix3 2.3·接续告知（只读检测·`workflow_chain_controller.rs` 本体 0-diff）：ensure_chain_run_record 会**复用**
    // 本 workflow 已有的 running/stopped 链记录断点续（completed 任务跳过、中断处的任务会重跑）——这里**镜像它的
    // 复用判据**(workflow_id+project_id+state∈{running,stopped})只读探一下，命中就给用户一句告知（不然静默重跑·
    // 用户不知情）。只读、不改链记录，检测放调用方。
    let resuming_prior_run = value
        .get("workflow_chain_runs")
        .and_then(Value::as_array)
        .is_some_and(|runs| {
            runs.iter().any(|run| {
                optional_string_from(run, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(run, "project_id").as_deref() == Some(pid.as_str())
                    && matches!(
                        optional_string_from(run, "state").as_deref(),
                        Some("running") | Some("stopped")
                    )
            })
        });
    if resuming_prior_run {
        push_unique(
            &mut warnings,
            "已接续上次中断的运行：中断处未完成的任务会重跑（已完成的跳过）。",
        );
    }
    let chain_run_id = ensure_chain_run_record(
        &mut value,
        &pid,
        workflow_id,
        &order_task_ids,
        max_nodes,
        &start_ts,
    )?;
    backup_workflow_state_file(path, &start_ts)?;
    append_chain_audit(
        &mut value,
        &chain_run_id,
        workflow_id,
        "workflow_chain_run_started",
        "ready",
        "running",
        &start_ts,
        "主管→worker 薄链驱动起链（圈固定测试项目，决策 2026-06-23）：按 depends_on 拓扑序逐任务过 S1 闸真跑、失败即停、可中断、有 runaway 上限。",
    )?;
    write_validated_workflow_state(path, &value)?;

    let mut dispatched = 0usize;
    let mut completed = 0usize;
    let mut skipped = 0usize;
    let mut steps: Vec<DirectorChainStep> = Vec::new();

    for task_id in &order_task_ids {
        let task = match tasks.iter().find(|task| &task.planned_task_id == task_id) {
            Some(task) => task,
            None => continue,
        };
        // 每任务边界重读（execute 会写文件；同时拿 stop_requested 最新值）。
        let mut current = read_workflow_state_value(path)?;
        // 断点续：本任务已 completed → 跳过。
        if chain_node_state(&current, &chain_run_id, task_id).as_deref() == Some("completed") {
            continue;
        }
        // 可中断（护栏②）：收到停链请求 → 在任务边界停（已完成任务保留、可断点续）。
        if chain_run_stop_requested(&current, &chain_run_id) {
            let ts = unix_timestamp_string();
            finalize_chain_run(&mut current, &chain_run_id, "stopped", &ts);
            append_chain_audit(
                &mut current,
                &chain_run_id,
                workflow_id,
                "workflow_chain_run_stopped",
                "running",
                "stopped",
                &ts,
                "收到停链请求，已在任务边界停下（已完成任务保留，可断点续）。",
            )?;
            write_validated_workflow_state(path, &current)?;
            return Ok(DirectorChainOutcome {
                total,
                dispatched,
                completed,
                skipped,
                chain_run_id,
                steps,
                director_summary: None,
                warnings,
                stopped_reason: Some("user_stop_requested".to_string()),
            });
        }
        // B1·修撒谎 filter：annotate 给**所有**任务（含 blocked/needs_binding）都设了 work_item+node
        // （c4_c6:1805/1807），所以旧的「(Some,Some)=>…,_=>continue」永不跳过、那句「被 guard 拦的跳过」
        // 是假的。改为**按授权状态过滤**：只派 status=="prepared"；blocked/needs_binding/其它 → 记 skipped、
        // 不进 execute（不在未授权任务上真起 codex）。
        if task.status != "prepared" {
            let ts = unix_timestamp_string();
            set_chain_node_state(
                &mut current,
                &chain_run_id,
                task_id,
                "skipped",
                None,
                Some(&format!(
                    "status={}（非 prepared，未授权派发）",
                    task.status
                )),
            );
            append_chain_audit(
                &mut current,
                &chain_run_id,
                workflow_id,
                "workflow_chain_node_skipped",
                "pending",
                "skipped",
                &ts,
                &format!(
                    "任务「{}」status={}（blocked/needs_binding 等，非 prepared），跳过不派发。",
                    task.title, task.status
                ),
            )?;
            write_validated_workflow_state(path, &current)?;
            skipped += 1;
            steps.push(DirectorChainStep {
                planned_task_id: task_id.clone(),
                title: task.title.clone(),
                state: "skipped".to_string(),
                report_summary: None,
                report_warning: None,
                report_status: None,
            });
            continue;
        }
        // prepared 理应有 node+work_item（annotate 无条件设）；防御：缺则记 skipped、不派。
        let (node_id, work_item_id) =
            match (task.workflow_node_id.clone(), task.work_item_id.clone()) {
                (Some(node), Some(work_item)) => (node, work_item),
                _ => {
                    let ts = unix_timestamp_string();
                    set_chain_node_state(
                        &mut current,
                        &chain_run_id,
                        task_id,
                        "skipped",
                        None,
                        Some("prepared 但缺 node/work_item（异常），跳过不派发"),
                    );
                    append_chain_audit(
                        &mut current,
                        &chain_run_id,
                        workflow_id,
                        "workflow_chain_node_skipped",
                        "pending",
                        "skipped",
                        &ts,
                        &format!(
                            "任务「{}」status=prepared 但缺 node/work_item，跳过。",
                            task.title
                        ),
                    )?;
                    write_validated_workflow_state(path, &current)?;
                    skipped += 1;
                    steps.push(DirectorChainStep {
                        planned_task_id: task_id.clone(),
                        title: task.title.clone(),
                        state: "skipped".to_string(),
                        report_summary: None,
                        report_warning: None,
                        report_status: None,
                    });
                    continue;
                }
            };
        // runaway 上限（护栏①）：只对真派发计数（skipped 不占额），超额 → 停链。
        if dispatched >= max_nodes {
            let ts = unix_timestamp_string();
            finalize_chain_run(&mut current, &chain_run_id, "stopped", &ts);
            append_chain_audit(
                &mut current,
                &chain_run_id,
                workflow_id,
                "workflow_chain_run_stopped",
                "running",
                "stopped",
                &ts,
                &format!("达到 runaway 上限（{max_nodes} 个任务），已停链。"),
            )?;
            write_validated_workflow_state(path, &current)?;
            return Ok(DirectorChainOutcome {
                total,
                dispatched,
                completed,
                skipped,
                chain_run_id,
                steps,
                director_summary: None,
                warnings,
                stopped_reason: Some(format!("runaway_cap_reached:{max_nodes}")),
            });
        }
        // 标 running + 审计 node-start。
        let ts_start = unix_timestamp_string();
        set_chain_node_state(&mut current, &chain_run_id, task_id, "running", None, None);
        append_chain_audit(
            &mut current,
            &chain_run_id,
            workflow_id,
            "workflow_chain_node_started",
            "pending",
            "running",
            &ts_start,
            &format!(
                "薄链驱动：派发任务「{}」（node {node_id} / work_item {work_item_id}）",
                task.title
            ),
        )?;
        write_validated_workflow_state(path, &current)?;
        dispatched += 1;

        // C1·每任务独立会话（session_creator=Some=生产主路径）：绑定面板映射时只有 `new` 任务会进
        // C1；`existing` 已在起链前由现成绑定命令精确绑到本 task 的 node + work_item。映射缺项直接拒，
        // 不得落回 role 节点的旧共用会话。没有映射的旧兼容路径仍维持原语义。
        let create_new_session = match task_session_bindings {
            Some(bindings) => match bindings.get(task_id) {
                Some(binding) if binding.session_choice == "new" => true,
                Some(binding) if binding.session_choice == "existing" => false,
                Some(binding) => {
                    return Err(format!(
                        "任务「{}」会话映射非法（{}）；不能派发。",
                        task.title, binding.session_choice
                    ));
                }
                None => {
                    return Err(format!(
                        "任务「{}」缺少会话映射；不能回落到共用旧会话。",
                        task.title
                    ));
                }
            },
            None => session_creator.is_some(),
        };
        if create_new_session {
            let creator = session_creator.ok_or_else(|| {
                format!(
                    "任务「{}」选了新会话，但 C1 建会话入口不可用；不能派发。",
                    task.title
                )
            })?;
            if let Err(session_error) = create_and_bind_task_session(
                path,
                index,
                project_root,
                workflow_id,
                &node_id,
                &work_item_id,
                task,
                creator,
            ) {
                let ts_fail = unix_timestamp_string();
                let mut failing = read_workflow_state_value(path)?;
                let task_node = format!(
                    "{}:node:task:{}",
                    task.scope.workflow_id,
                    stable_id(task_id)
                );
                set_chain_node_state(
                    &mut failing,
                    &chain_run_id,
                    task_id,
                    "failed",
                    None,
                    Some(&session_error),
                );
                update_node_state_for_id(&mut failing, &task_node, "failed", &ts_fail)?;
                append_chain_audit(
                    &mut failing,
                    &chain_run_id,
                    workflow_id,
                    "workflow_chain_node_failed",
                    "running",
                    "failed",
                    &ts_fail,
                    &format!(
                        "薄链驱动：任务「{}」新建会话失败即停——{session_error}",
                        task.title
                    ),
                )?;
                finalize_chain_run(&mut failing, &chain_run_id, "failed", &ts_fail);
                append_chain_audit(
                    &mut failing,
                    &chain_run_id,
                    workflow_id,
                    "workflow_chain_run_failed",
                    "running",
                    "failed",
                    &ts_fail,
                    &format!(
                        "任务「{}」新建会话失败，已停链（失败即停·不回落共用会话）：{session_error}",
                        task.title
                    ),
                )?;
                write_validated_workflow_state(path, &failing)?;
                steps.push(DirectorChainStep {
                    planned_task_id: task_id.clone(),
                    title: task.title.clone(),
                    state: "failed".to_string(),
                    report_summary: None,
                    report_warning: None,
                    report_status: None,
                });
                return Ok(DirectorChainOutcome {
                    total,
                    dispatched,
                    completed,
                    skipped,
                    chain_run_id,
                    steps,
                    director_summary: None,
                    warnings,
                    stopped_reason: Some(format!("fail_stop:session_create:{}", task.title)),
                });
            }
        }

        // 真派发：复用 gated 的 _at（S1 闸 + 沙箱 + resume 会话），**本体不动**。
        let request = ProjectWorkflowNodeRunRequest {
            project_root: project_root.to_string(),
            node_id,
            work_item_id,
            workflow_id: Some(workflow_id.to_string()),
        };
        let mut outcome =
            execute_project_workflow_node_at(path, index, readback_db_path, runner, &request);

        // 2.4 flaky 自动重试一次：仅 tier-1 偶发早退（exit≠0·非 timeout·非沙箱/gate·记忆 real-codex-run-flaky）
        // 原地重试一次；越权被拒/闸拦/超时按原语义不 retry（is_tier1_early_exit 只认早退特征）。**不循环**。
        // 首次失败已把 work_item 推离 ready_to_dispatch → 重试前走**现成合法跳转**（failed→needs_changes→
        // ready_to_dispatch）复位；复位不成（如非默认工作流·update_work_item_state_at 限默认）则不重试。
        if is_tier1_early_exit(&outcome)
            && reset_work_item_for_retry(path, project_root, &request.work_item_id)
        {
            push_unique(
                &mut warnings,
                &format!(
                    "任务「{}」worker 偶发早退（exit≠0 无输出），已自动重试一次。",
                    task.title
                ),
            );
            outcome =
                execute_project_workflow_node_at(path, index, readback_db_path, runner, &request);
        }

        // 重读（execute 写过文件，避免覆盖它的写入）。
        let mut after = read_workflow_state_value(path)?;
        let ts_done = unix_timestamp_string();
        // 2.3 尾·加法小缝：对应**任务级节点**（prepare 按 source_ref=planned_task_id 建的 {wf}:node:task:{stable_id}）
        // 顺带把 state 刷成 completed/failed，让画布进度跟链走。update_node_state_for_id 找不到即 no-op（无任务级
        // 节点的旧链/resume 路径不受影响）——**不碰链判决体**（set_chain_node_state / 停链 / finalize 全原样）。
        let task_level_node_id = format!(
            "{}:node:task:{}",
            task.scope.workflow_id,
            stable_id(task_id)
        );
        match outcome {
            Ok(result) if result.dispatch.state == "completed" => {
                let dispatch_id = result.dispatch.dispatch_id.clone();
                let last_message_full = result
                    .dispatch
                    .last_message_path
                    .as_deref()
                    .and_then(|last_message_path| std::fs::read_to_string(last_message_path).ok())
                    .unwrap_or_default();
                if worker_report::help_signal_from_raw(&last_message_full).is_some() {
                    let report_outcome = worker_report::consume_worker_report_after_completion(
                        path,
                        project_root,
                        &result.dispatch.project_id,
                        &result.dispatch.workflow_id,
                        &result.dispatch.node_id,
                        &result.dispatch.work_item_id,
                        Some(dispatch_id.as_str()),
                        &task.scope.target_role,
                        &task.title,
                        &last_message_full,
                    );
                    let help_signal = report_outcome.help_signal.clone().unwrap_or_else(|| {
                        worker_report::WorkerReportHelpSignal {
                            status: "suspected_blocked".to_string(),
                            summary: "worker 疑似求助·主管必看".to_string(),
                            open_issues: vec![],
                            permission_requests: vec![],
                            direction_risks: vec![],
                            follow_up_suggestions: vec![],
                        }
                    });
                    let mut after_help = read_workflow_state_value(path)?;
                    let help_message = worker_help_message(&help_signal);
                    set_chain_node_state(
                        &mut after_help,
                        &chain_run_id,
                        task_id,
                        "waiting_decision",
                        Some(&dispatch_id),
                        Some(&help_message),
                    );
                    update_node_state_for_id(
                        &mut after_help,
                        &task_level_node_id,
                        "waiting_decision",
                        &ts_done,
                    )?;
                    append_chain_audit(
                        &mut after_help,
                        &chain_run_id,
                        workflow_id,
                        "workflow_chain_node_waiting_decision",
                        "running",
                        "waiting_decision",
                        &ts_done,
                        &format!("薄链驱动：任务「{}」worker 求助，待主管决策——{help_message}", task.title),
                    )?;
                    finalize_chain_run(&mut after_help, &chain_run_id, "waiting_decision", &ts_done);
                    append_chain_audit(
                        &mut after_help,
                        &chain_run_id,
                        workflow_id,
                        "workflow_chain_run_waiting_decision",
                        "running",
                        "waiting_decision",
                        &ts_done,
                        &format!("任务「{}」worker 求助，链已停在 waiting_decision。", task.title),
                    )?;
                    write_validated_workflow_state(path, &after_help)?;
                    steps.push(DirectorChainStep {
                        planned_task_id: task_id.clone(),
                        title: task.title.clone(),
                        state: "waiting_decision".to_string(),
                        report_summary: Some(help_signal.summary),
                        report_warning: report_outcome.report_warning,
                        report_status: report_outcome.report_status,
                    });
                    return Ok(DirectorChainOutcome {
                        total,
                        dispatched,
                        completed,
                        skipped,
                        chain_run_id,
                        steps,
                        director_summary: None,
                        warnings,
                        stopped_reason: Some(format!("waiting_decision:worker_help:{}", task.title)),
                    });
                }
                let parsed_report = worker_report::parse_worker_report(&last_message_full);
                let report_outcome = worker_report::consume_worker_report_after_completion(
                    path,
                    project_root,
                    &result.dispatch.project_id,
                    &result.dispatch.workflow_id,
                    &result.dispatch.node_id,
                    &result.dispatch.work_item_id,
                    Some(dispatch_id.as_str()),
                    &task.scope.target_role,
                    &task.title,
                    &last_message_full,
                );
                let screen = director_final_screen(task, parsed_report.as_ref());
                let attempts_used = chain_node_usize_field(
                    &read_workflow_state_value(path)?,
                    &chain_run_id,
                    task_id,
                    "director_rework_attempts",
                );
                let remaining_budget =
                    DIRECTOR_FINAL_REWORK_BUDGET.saturating_sub(attempts_used);
                let final_decision = if screen.is_green() {
                    Ok(DirectorFinalMark {
                        decision: DirectorFinalMarkDecision::Completed,
                        reason: "主管终标·确定性初筛①-⑤全绿，零 LM 直过。".to_string(),
                    })
                } else {
                    final_marker.final_mark(&DirectorFinalMarkContext {
                        project_root: project_root.to_string(),
                        workflow_id: workflow_id.to_string(),
                        task_title: task.title.clone(),
                        task_goal: task.task_goal.clone(),
                        acceptance_criteria: task.acceptance_criteria.clone(),
                        report_status: screen.report_status.clone(),
                        acceptance_status: screen.acceptance_status.clone(),
                        evidence_refs: screen.evidence_refs.clone(),
                        direction_risks: screen.direction_risks.clone(),
                        yellow_reasons: screen.yellow_reasons.clone(),
                        last_message_tail: tail_chars(&last_message_full, 800),
                        rework_budget_remaining: remaining_budget,
                    })
                };

                match final_decision {
                    Ok(mark) if mark.decision == DirectorFinalMarkDecision::Completed => {
                        let mut after_mark = read_workflow_state_value(path)?;
                        set_chain_node_state(
                            &mut after_mark,
                            &chain_run_id,
                            task_id,
                            "completed",
                            Some(&dispatch_id),
                            None,
                        );
                        update_node_state_for_id(
                            &mut after_mark,
                            &task_level_node_id,
                            "completed",
                            &ts_done,
                        )?;
                        let final_event = if screen.is_green() {
                            "workflow_chain_node_director_deterministic_completed"
                        } else {
                            "workflow_chain_node_director_lm_completed"
                        };
                        append_chain_audit(
                            &mut after_mark,
                            &chain_run_id,
                            workflow_id,
                            final_event,
                            "running",
                            "completed",
                            &ts_done,
                            &format!("主管终标通过任务「{}」：{}", task.title, mark.reason),
                        )?;
                        append_chain_audit(
                            &mut after_mark,
                            &chain_run_id,
                            workflow_id,
                            "workflow_chain_node_completed",
                            "running",
                            "completed",
                            &ts_done,
                            &format!(
                                "薄链驱动：任务「{}」主管终标 completed（dispatch {dispatch_id}）",
                                task.title
                            ),
                        )?;
                        write_validated_workflow_state(path, &after_mark)?;
                        completed += 1;
                        steps.push(DirectorChainStep {
                            planned_task_id: task_id.clone(),
                            title: task.title.clone(),
                            state: "completed".to_string(),
                            report_summary: report_outcome.report_summary,
                            report_warning: report_outcome.report_warning,
                            report_status: report_outcome.report_status,
                        });
                    }
                    Ok(mark) => {
                        if remaining_budget > 0 {
                            let reset_ok = reset_work_item_for_director_rework(
                                path,
                                project_root,
                                &request.work_item_id,
                            );
                            let mut after_rework = read_workflow_state_value(path)?;
                            set_chain_node_state(
                                &mut after_rework,
                                &chain_run_id,
                                task_id,
                                "needs_rework",
                                Some(&dispatch_id),
                                Some(&mark.reason),
                            );
                            set_chain_node_usize_field(
                                &mut after_rework,
                                &chain_run_id,
                                task_id,
                                "director_rework_attempts",
                                attempts_used + 1,
                            );
                            set_chain_node_usize_field(
                                &mut after_rework,
                                &chain_run_id,
                                task_id,
                                "director_rework_budget",
                                DIRECTOR_FINAL_REWORK_BUDGET,
                            );
                            update_node_state_for_id(
                                &mut after_rework,
                                &task_level_node_id,
                                "needs_rework",
                                &ts_done,
                            )?;
                            append_chain_audit(
                                &mut after_rework,
                                &chain_run_id,
                                workflow_id,
                                "workflow_chain_node_needs_rework",
                                "running",
                                "needs_rework",
                                &ts_done,
                                &format!(
                                    "主管终标退回任务「{}」：{}{}",
                                    task.title,
                                    mark.reason,
                                    if reset_ok {
                                        "（已复位为可重做）"
                                    } else {
                                        "（复位到可重做失败，待人工处理）"
                                    }
                                ),
                            )?;
                            finalize_chain_run(&mut after_rework, &chain_run_id, "stopped", &ts_done);
                            append_chain_audit(
                                &mut after_rework,
                                &chain_run_id,
                                workflow_id,
                                "workflow_chain_run_stopped",
                                "running",
                                "stopped",
                                &ts_done,
                                &format!(
                                    "任务「{}」主管终标退回 needs_rework，已消耗返工预算 {}/{}。",
                                    task.title,
                                    attempts_used + 1,
                                    DIRECTOR_FINAL_REWORK_BUDGET
                                ),
                            )?;
                            write_validated_workflow_state(path, &after_rework)?;
                            steps.push(DirectorChainStep {
                                planned_task_id: task_id.clone(),
                                title: task.title.clone(),
                                state: "needs_rework".to_string(),
                                report_summary: report_outcome.report_summary,
                                report_warning: report_outcome.report_warning,
                                report_status: report_outcome.report_status,
                            });
                            return Ok(DirectorChainOutcome {
                                total,
                                dispatched,
                                completed,
                                skipped,
                                chain_run_id,
                                steps,
                                director_summary: None,
                                warnings,
                                stopped_reason: Some(format!(
                                    "needs_rework:director_final_mark:{}",
                                    task.title
                                )),
                            });
                        }
                        let mut waiting = read_workflow_state_value(path)?;
                        let wait_message = format!(
                            "主管终标退回但返工预算已耗尽，待人工决策：{}",
                            mark.reason
                        );
                        set_chain_node_state(
                            &mut waiting,
                            &chain_run_id,
                            task_id,
                            "waiting_decision",
                            Some(&dispatch_id),
                            Some(&wait_message),
                        );
                        update_node_state_for_id(
                            &mut waiting,
                            &task_level_node_id,
                            "waiting_decision",
                            &ts_done,
                        )?;
                        append_chain_audit(
                            &mut waiting,
                            &chain_run_id,
                            workflow_id,
                            "workflow_chain_node_waiting_decision",
                            "running",
                            "waiting_decision",
                            &ts_done,
                            &format!("任务「{}」主管退回预算耗尽：{wait_message}", task.title),
                        )?;
                        finalize_chain_run(&mut waiting, &chain_run_id, "waiting_decision", &ts_done);
                        append_chain_audit(
                            &mut waiting,
                            &chain_run_id,
                            workflow_id,
                            "workflow_chain_run_waiting_decision",
                            "running",
                            "waiting_decision",
                            &ts_done,
                            &format!("任务「{}」返工预算耗尽，链已停在 waiting_decision。", task.title),
                        )?;
                        write_validated_workflow_state(path, &waiting)?;
                        steps.push(DirectorChainStep {
                            planned_task_id: task_id.clone(),
                            title: task.title.clone(),
                            state: "waiting_decision".to_string(),
                            report_summary: report_outcome.report_summary,
                            report_warning: report_outcome.report_warning,
                            report_status: report_outcome.report_status,
                        });
                        return Ok(DirectorChainOutcome {
                            total,
                            dispatched,
                            completed,
                            skipped,
                            chain_run_id,
                            steps,
                            director_summary: None,
                            warnings,
                            stopped_reason: Some(format!(
                                "waiting_decision:director_final_mark_budget_exhausted:{}",
                                task.title
                            )),
                        });
                    }
                    Err(error) => {
                        let mut waiting = read_workflow_state_value(path)?;
                        let wait_message =
                            format!("主管终标 LM 不可用，保守待人工决策：{error}");
                        set_chain_node_state(
                            &mut waiting,
                            &chain_run_id,
                            task_id,
                            "waiting_decision",
                            Some(&dispatch_id),
                            Some(&wait_message),
                        );
                        update_node_state_for_id(
                            &mut waiting,
                            &task_level_node_id,
                            "waiting_decision",
                            &ts_done,
                        )?;
                        append_chain_audit(
                            &mut waiting,
                            &chain_run_id,
                            workflow_id,
                            "workflow_chain_node_waiting_decision",
                            "running",
                            "waiting_decision",
                            &ts_done,
                            &format!("任务「{}」主管终标不可用：{wait_message}", task.title),
                        )?;
                        finalize_chain_run(&mut waiting, &chain_run_id, "waiting_decision", &ts_done);
                        append_chain_audit(
                            &mut waiting,
                            &chain_run_id,
                            workflow_id,
                            "workflow_chain_run_waiting_decision",
                            "running",
                            "waiting_decision",
                            &ts_done,
                            &format!("任务「{}」主管终标 LM 断供，链已停在 waiting_decision。", task.title),
                        )?;
                        write_validated_workflow_state(path, &waiting)?;
                        steps.push(DirectorChainStep {
                            planned_task_id: task_id.clone(),
                            title: task.title.clone(),
                            state: "waiting_decision".to_string(),
                            report_summary: report_outcome.report_summary,
                            report_warning: report_outcome.report_warning,
                            report_status: report_outcome.report_status,
                        });
                        return Ok(DirectorChainOutcome {
                            total,
                            dispatched,
                            completed,
                            skipped,
                            chain_run_id,
                            steps,
                            director_summary: None,
                            warnings,
                            stopped_reason: Some(format!(
                                "waiting_decision:director_final_mark_unavailable:{}",
                                task.title
                            )),
                        });
                    }
                }
            }
            // 失败即停（护栏·不自动重试/不跳过，防在老失败任务上打转）。
            other => {
                let fail_msg = match &other {
                    Ok(result) => {
                        // 质量债·超时可辨：dispatch.state 恒 "failed"（write_failed_dispatch 设计），
                        // timed_out 语义在 classify 落进 dispatch.warnings 的现成 "timeout" 信号——
                        // 带进停因（·timed_out 标记），反馈边/已完成事实收集按它判，不新造分类。
                        let timed_out_tag = if result
                            .dispatch
                            .warnings
                            .iter()
                            .any(|warning| warning == "timeout")
                        {
                            "·timed_out"
                        } else {
                            ""
                        };
                        format!(
                            "worker 派发未完成（state={}{timed_out_tag}）",
                            result.dispatch.state
                        )
                    }
                    Err(error) => error.clone(),
                };
                set_chain_node_state(
                    &mut after,
                    &chain_run_id,
                    task_id,
                    "failed",
                    None,
                    Some(&fail_msg),
                );
                update_node_state_for_id(&mut after, &task_level_node_id, "failed", &ts_done)?;
                append_chain_audit(
                    &mut after,
                    &chain_run_id,
                    workflow_id,
                    "workflow_chain_node_failed",
                    "running",
                    "failed",
                    &ts_done,
                    &format!("薄链驱动：任务「{}」失败即停——{fail_msg}", task.title),
                )?;
                finalize_chain_run(&mut after, &chain_run_id, "failed", &ts_done);
                append_chain_audit(
                    &mut after,
                    &chain_run_id,
                    workflow_id,
                    "workflow_chain_run_failed",
                    "running",
                    "failed",
                    &ts_done,
                    &format!(
                        "任务「{}」失败，已停链（失败即停、不自动重试）：{fail_msg}",
                        task.title
                    ),
                )?;
                write_validated_workflow_state(path, &after)?;
                steps.push(DirectorChainStep {
                    planned_task_id: task_id.clone(),
                    title: task.title.clone(),
                    state: "failed".to_string(),
                    report_summary: None,
                    report_warning: None,
                    report_status: None,
                });
                return Ok(DirectorChainOutcome {
                    total,
                    dispatched,
                    completed,
                    skipped,
                    chain_run_id,
                    steps,
                    director_summary: None,
                    warnings,
                    stopped_reason: Some(format!("fail_stop:node_error:{}:{fail_msg}", task.title)),
                });
            }
        }
    }

    // 收尾：completed + 审计「链完成」。
    let ts_close = unix_timestamp_string();
    let mut closing = read_workflow_state_value(path)?;
    finalize_chain_run(&mut closing, &chain_run_id, "completed", &ts_close);
    append_chain_audit(
        &mut closing,
        &chain_run_id,
        workflow_id,
        "workflow_chain_run_completed",
        "running",
        "completed",
        &ts_close,
        "主管→worker 薄链驱动完成：所有 prepared 任务按 depends_on 序真派发成功。",
    )?;
    write_validated_workflow_state(path, &closing)?;
    let summary_context = DirectorWorkflowSummaryContext {
        project_root: project_root.to_string(),
        workflow_id: workflow_id.to_string(),
        chain_run_id: chain_run_id.clone(),
        total,
        dispatched,
        completed,
        skipped,
        steps: steps.clone(),
        warnings: warnings.clone(),
    };
    let director_summary = match summary_generator.summarize_chain(&summary_context) {
        Ok(summary) => {
            let capture_result = capture_director_summary_candidate(
                path,
                project_root,
                workflow_id,
                &chain_run_id,
                &summary,
                &ts_close,
            );
            let candidate_note = match capture_result {
                Ok(output) => output
                    .capture_event
                    .candidate_key
                    .as_deref()
                    .map(|candidate_key| format!("候选：{candidate_key}"))
                    .unwrap_or_else(|| "未生成候选".to_string()),
                Err(error) => {
                    push_unique(
                        &mut warnings,
                        &format!("director_summary_capture_failed:{error}"),
                    );
                    "候选生成失败（已软着陆）".to_string()
                }
            };
            match read_workflow_state_value(path).and_then(|mut summary_state| {
                append_chain_audit(
                    &mut summary_state,
                    &chain_run_id,
                    workflow_id,
                    "workflow_chain_director_summary",
                    "completed",
                    "completed",
                    &ts_close,
                    &format!("主管链末总结已生成：{}；{candidate_note}", summary.summary),
                )?;
                write_validated_workflow_state(path, &summary_state)
            }) {
                Ok(_) => {}
                Err(error) => push_unique(
                    &mut warnings,
                    &format!("director_summary_audit_failed:{error}"),
                ),
            }
            Some(summary)
        }
        Err(error) => {
            push_unique(
                &mut warnings,
                &format!("director_summary_unavailable:{error}"),
            );
            None
        }
    };
    Ok(DirectorChainOutcome {
        total,
        dispatched,
        completed,
        skipped,
        chain_run_id,
        steps,
        director_summary,
        warnings,
        stopped_reason: None,
    })
}

// C1·建+绑一条任务专属会话（先生后绑单次路径·existing 绑定机器·别新造第二套）。成功返回新会话 thread_id；
// 失败返回人话（供给类经 create_initialized_session 已带 fix8 前缀）。成功后把 target_session_id 回填任务包
// artifact（C0 差量 §5.1 的 C1 项·加法一处）。绑定审计走 bind_workflow_node_codex_session_for_index_at 自带的
// 事件族（不新开）；cwd/沙箱写死固定测试项目（在 creator 内·不可参数化）。
fn create_and_bind_task_session(
    path: &std::path::Path,
    index: &Value,
    project_root: &str,
    workflow_id: &str,
    node_id: &str,
    work_item_id: &str,
    task: &ProjectDirectorPlannedTask,
    creator: &dyn JiaobanNewSessionCreator,
) -> Result<String, String> {
    // 会话初始化消息 = 以任务命名（截断安全·智能体页列表可辨），只叫它「就位」，别改文件（任务随后经任务包发）。
    let clipped_title: String = task.title.chars().take(80).collect();
    let init_text = format!(
        "交办任务专用会话：本会话只承接任务「{clipped_title}」（工作流 {workflow_id}）。现在先别改任何文件，回复「已就位」即可；任务详情随后经任务包发来。"
    );
    let thread_id = creator
        .create_initialized_session(&init_text, "director_chain")
        .map_err(|error| format!("新建会话失败：{error}"))?;
    // 绑到本任务角色节点（existing 同款机器）；只对「会话落 codex 自家 sqlite 晚一拍」的可见性时差重试。
    let bind_request = WorkflowNodeSessionBindRequest {
        project_root: project_root.to_string(),
        node_id: node_id.to_string(),
        work_item_id: Some(work_item_id.to_string()),
        thread_id: thread_id.clone(),
    };
    let bind_started = std::time::Instant::now();
    loop {
        match bind_workflow_node_codex_session_for_index_at(path, index, &bind_request) {
            Ok(_) => break,
            Err(error)
                if error.contains("会话不在当前索引内")
                    && bind_started.elapsed().as_millis()
                        < JIAOBAN_NEW_SESSION_BIND_VISIBILITY_BUDGET_MS =>
            {
                std::thread::sleep(std::time::Duration::from_millis(
                    JIAOBAN_NEW_SESSION_POLL_INTERVAL_MS,
                ));
            }
            Err(error) => {
                return Err(format!(
                    "新会话已建（thread {thread_id}）但绑定失败：{error}"
                ));
            }
        }
    }
    // target_session_id 回填任务包 artifact（找不到即 no-op·防御式不崩链）。
    let mut value = read_workflow_state_value(path)?;
    if set_task_artifact_target_session_id(&mut value, task, &thread_id) {
        write_validated_workflow_state(path, &value)?;
    }
    Ok(thread_id)
}

// C1·把新会话 thread_id 回填任务包 artifact 的 target_session_id。找到并回填 → true；缺 artifact_id/artifact
// → false（no-op·不崩）。**只写 target_session_id 一个字段**，不碰 artifact 其它字段/判决体。
fn set_task_artifact_target_session_id(
    value: &mut Value,
    task: &ProjectDirectorPlannedTask,
    thread_id: &str,
) -> bool {
    let artifact_id = match task.task_package_id.as_deref() {
        Some(id) => id,
        None => return false,
    };
    if let Some(artifacts) = value.get_mut("artifacts").and_then(Value::as_array_mut) {
        for artifact in artifacts.iter_mut() {
            if optional_string_from(artifact, "artifact_id").as_deref() == Some(artifact_id) {
                artifact["target_session_id"] = Value::String(thread_id.to_string());
                return true;
            }
        }
    }
    false
}

fn validate_task_session_bindings(
    tasks: &[ProjectDirectorPlannedTask],
    bindings: &[ProjectDirectorTaskSessionBinding],
    index: &Value,
) -> Result<std::collections::BTreeMap<String, ProjectDirectorTaskSessionBinding>, String> {
    let expected: std::collections::BTreeSet<String> =
        tasks.iter().map(|task| task.planned_task_id.clone()).collect();
    if expected.len() != tasks.len() {
        return Err("任务清单编号重复，不能确认会话映射。请重新出方案。".to_string());
    }

    let mut mapped = std::collections::BTreeMap::new();
    for binding in bindings {
        if mapped
            .insert(binding.planned_task_id.clone(), binding.clone())
            .is_some()
        {
            return Err("同一任务被重复选择会话，不能确认映射。".to_string());
        }
    }
    let actual: std::collections::BTreeSet<String> = mapped.keys().cloned().collect();
    if actual != expected {
        return Err("任务会话映射和当前任务清单不一致（缺项或多项）；请重新出方案。".to_string());
    }

    for binding in mapped.values_mut() {
        match binding.session_choice.as_str() {
            "new" => {
                if binding
                    .session_id
                    .as_deref()
                    .is_some_and(|session_id| !session_id.trim().is_empty())
                {
                    return Err("新会话任务不应带入已有会话；请重新选择。".to_string());
                }
                binding.session_id = None;
            }
            "existing" => {
                let session_id = binding
                    .session_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|session_id| !session_id.is_empty())
                    .ok_or_else(|| "有任务没有选好要沿用的对话。".to_string())?;
                if find_index_thread_or_sqlite(index, session_id).is_none() {
                    return Err("选中的已有对话已不可用；请重新选择。".to_string());
                }
                binding.session_id = Some(session_id.to_string());
            }
            _ => return Err("会话选择不认识；请重新选择新会话或已有对话。".to_string()),
        }
    }
    Ok(mapped)
}

fn bind_existing_task_sessions(
    path: &std::path::Path,
    index: &Value,
    project_root: &str,
    tasks: &[ProjectDirectorPlannedTask],
    task_session_bindings: &std::collections::BTreeMap<String, ProjectDirectorTaskSessionBinding>,
) -> Result<(), String> {
    for task in tasks {
        if task.status != "prepared" {
            continue;
        }
        let binding = task_session_bindings.get(&task.planned_task_id).ok_or_else(|| {
            format!("任务「{}」缺少会话映射，不能继续。", task.title)
        })?;
        if binding.session_choice != "existing" {
            continue;
        }
        let node_id = task.workflow_node_id.as_deref().ok_or_else(|| {
            format!("任务「{}」缺少执行节点，不能绑定已有对话。", task.title)
        })?;
        let work_item_id = task.work_item_id.as_deref().ok_or_else(|| {
            format!("任务「{}」缺少工作项，不能绑定已有对话。", task.title)
        })?;
        let session_id = binding.session_id.as_deref().ok_or_else(|| {
            format!("任务「{}」没有可用的已有对话。", task.title)
        })?;
        bind_workflow_node_codex_session_for_index_at(
            path,
            index,
            &WorkflowNodeSessionBindRequest {
                project_root: project_root.to_string(),
                node_id: node_id.to_string(),
                work_item_id: Some(work_item_id.to_string()),
                thread_id: session_id.to_string(),
            },
        )
        .map_err(|error| format!("任务「{}」绑定已有对话失败：{error}", task.title))?;

        let mut value = read_workflow_state_value(path)?;
        if !set_task_artifact_target_session_id(&mut value, task, session_id) {
            return Err(format!(
                "任务「{}」已有对话已绑定，但任务包没有可回填的位置；已停下，未回落。",
                task.title
            ));
        }
        write_validated_workflow_state(path, &value)?;
    }
    Ok(())
}

// ===== C1·生产起链命令（app 内 async 起整条主管链；停/进度复用现成命令）=====
// 收前端回传的「已审 planned_tasks」(preview→用户审→prepare 返回那份·含 depends_on) → spawn_blocking 调现成
// run_director_task_chain（每节点过 S1 闸·入口 require_test_project_path_lock 圈测试项目）→ 返回 outcome。
// 死线：① async+spawn_blocking（同步会冻 UI + 停链抢不到线程 = 可中断形同虚设，照 start_project_workflow_chain
// 范本）② 不重跑 LM（C1 不持有 director、只收 planned_tasks——跑的就是用户审过的计划）③ 不自建更松入口（圈
// 测试项目全靠 driver 入口的 require_test_project_path_lock）④ 停/进度复用 stop_project_workflow_chain /
// get_project_workflow_chain_status（0 新命令）。gate/沙箱/execute/prepare/controller/driver 本体 0-diff。
#[derive(serde::Deserialize)]
pub(crate) struct StartProjectDirectorChainRequest {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    // 前端从 prepare 返回回传的已审计划（含 depends_on/已 annotate）——绝不在此重跑 LM。
    pub(crate) planned_tasks: Vec<ProjectDirectorPlannedTask>,
    #[serde(default)]
    pub(crate) max_nodes: Option<usize>,
}

#[tauri::command]
async fn start_project_director_chain(
    request: StartProjectDirectorChainRequest,
    state: tauri::State<'_, AppState>,
) -> Result<DirectorChainOutcome, String> {
    // index/path 在 await 前从 state 取（State 不能跨进 'static 闭包）——同 start_project_workflow_chain 范本。
    let path = state.workflow_state_path.clone();
    let index = read_index(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        // 2.5 起链前复查授权仍 active（批与跑之间可能被撤/过期）——C1 直接收 planned_tasks 起链，尤需此复查。
        require_active_authorization(&path, &request.project_root, &request.workflow_id)?;
        let readback_db_path = codex_db::default_state_db_path();
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        // C1·生产主路径：每任务先生后绑建专属会话（真 relay 单次路径）。拐杖退役=此处不再走旧共用绑定。
        run_director_task_chain_with_session_creator(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &request.project_root,
            &request.workflow_id,
            &request.planned_tasks,
            request.max_nodes.unwrap_or(50),
            &ManualRelayJiaobanNewSessionCreator,
        )
    })
    .await
    .map_err(|error| format!("主管链执行线程异常：{error}"))?
}

#[tauri::command]
async fn apply_project_director_failed_action(
    request: ProjectDirectorFailedActionRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectDirectorFailedActionOutcome, String> {
    let path = state.workflow_state_path.clone();
    let index = read_index(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let readback_db_path = codex_db::default_state_db_path();
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        run_project_director_failed_action_with_session_creator(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &request,
            &ManualRelayJiaobanNewSessionCreator,
        )
    })
    .await
    .map_err(|error| format!("四选一节点处置线程异常：{error}"))?
}

// ===== P1·角色循环「授权后自动推进」编排命令（件 B + 件 C-1）=====
// 查 active 方案授权（人闸·不创建不跳过）→ 主管 LM 拆任务（preview）→ prepare → 件 C-1 分流（没绑/越界/无可派 → 停 +
// 可见）→ prepared 出来就跑 worker 链（run_director_task_chain·四护栏·入口 path-lock 圈测试项目）。
// 复用现成 preview/prepare/chain **本体 0-diff**，只新增编排；每阶段审计进 audit_events（canonical 形）。
#[derive(serde::Deserialize)]
pub(crate) struct AutoAdvanceAuthorizedRoleLoopRequest {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    #[serde(default)]
    pub(crate) max_nodes: Option<usize>,
    #[serde(default)]
    pub(crate) actor_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct AutoAdvanceRoleLoopOutcome {
    // "ran" | "needs_binding" | "blocked" | "no_dispatchable"
    pub(crate) stage: String,
    pub(crate) planned_task_count: usize,
    pub(crate) prepared_count: usize,
    pub(crate) needs_binding_count: usize,
    pub(crate) blocked_count: usize,
    pub(crate) message: String,
    pub(crate) chain_outcome: Option<DirectorChainOutcome>,
    pub(crate) stop_reason: Option<String>,
    // 开工前逐任务会话面板：复用 needs_binding 阶段，但只在「拆完、尚未 prepare」时为 true。
    // serde 加法，旧前端可忽略；不参与授权或派发判定。
    #[serde(default)]
    pub(crate) task_session_binding_required: bool,
    // 前端在链停后的用户处置要原样回传目标任务；只读回显，不参与派发或授权判断。
    #[serde(default)]
    pub(crate) planned_tasks: Vec<ProjectDirectorPlannedTask>,
    // fix3 2.1：非致命提示（如角色钳位「任务 X 角色 Y 不在授权名单，已按 codex-dev 执行」）。
    // 加法字段·前端可忽略；None 路（所批即所跑·approved 已在预拆钳过）不重复钳，此处为空。
    #[serde(default)]
    pub(crate) warnings: Vec<String>,
}

// 编排级审计：append 进 audit_events（canonical 形·与初始化/c4 事件同字段），读改写一次（不与复用 fn 的写交错）。
fn append_role_loop_auto_advance_audit(
    path: &std::path::Path,
    workflow_id: &str,
    actor_id: &str,
    event_type: &str,
    reason: &str,
) -> Result<(), String> {
    let mut value = read_workflow_state_value(path)?;
    let ts = unix_timestamp_string();
    array_mut(&mut value, "audit_events")?.push(serde_json::json!({
        "event_id": format!("role-loop-auto-advance:{event_type}:{ts}"),
        "event_type": event_type,
        "target_ref": workflow_id,
        "actor_ref": actor_id,
        "source_kind": "role_loop_auto_advance",
        "permission_level": "workflow_event_record",
        "created_at": ts,
        "reason": reason,
    }));
    write_validated_workflow_state(path, &value)
}

// 2.5 起链前复查方案授权仍 active（批与跑之间可能被撤/过期）；无 active → 拒。复用 active 授权解析口径。
// 调用处加缝——不改 run_director_task_chain / 授权 store 本体。
fn require_active_authorization(
    path: &std::path::Path,
    project_root: &str,
    workflow_id: &str,
) -> Result<(), String> {
    let timestamp_ms = unix_timestamp_ms();
    let pid = project_id(project_root);
    let store = plan_authorization_store::load_store(path, timestamp_ms)?;
    let active = store.authorizations.iter().any(|authorization| {
        authorization.project_id == pid
            && authorization.workflow_id == workflow_id
            && authorization.status == PlanAuthorizationStatus::Active
            && authorization
                .expires_at_ms
                .is_none_or(|expires_at_ms| expires_at_ms > timestamp_ms)
    });
    if active {
        Ok(())
    } else {
        Err(
            "方案授权已失效或被撤销（批与跑之间被撤/过期）；不能起链。请重新确认方案 + 全局边界复核。"
                .to_string(),
        )
    }
}

// 编排内层（同步·spawn_blocking 里调；可单测·stub runner）。
fn run_auto_advance_authorized_role_loop(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    project_root: &str,
    workflow_id: &str,
    actor_id: &str,
    max_nodes: usize,
    approved_planned_tasks: Option<&[ProjectDirectorPlannedTask]>,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    // 质量债·超时反馈边：预算**写死 1**（要改另拍）——薄壳保签名（既有调用点/lib.rs 测试 0 改动）。
    // C1·mode-aware：本公有壳 = 拐杖路（session_creator=None）——手动挡/existing/旧测试照走预绑 resume·守卫零改。
    run_auto_advance_authorized_role_loop_with_timeout_budget(
        path,
        index,
        readback_db_path,
        runner,
        director,
        project_root,
        workflow_id,
        actor_id,
        max_nodes,
        approved_planned_tasks,
        1,
        None,
        false,
        None,
    )
}

// C1·自动/新对话免管路主入口（canon 2026-07-09）：每任务经现成先生后绑建专属新会话。独立[接着跑]（无
// session_choice·天然新对话模式）走这条。lib.rs 旧测试仍调公有壳（None）→ 守卫/拐杖测零改。
#[allow(clippy::too_many_arguments)]
fn run_auto_advance_authorized_role_loop_with_session_creator(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    project_root: &str,
    workflow_id: &str,
    actor_id: &str,
    max_nodes: usize,
    approved_planned_tasks: Option<&[ProjectDirectorPlannedTask]>,
    session_creator: &dyn JiaobanNewSessionCreator,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    run_auto_advance_authorized_role_loop_with_timeout_budget(
        path,
        index,
        readback_db_path,
        runner,
        director,
        project_root,
        workflow_id,
        actor_id,
        max_nodes,
        approved_planned_tasks,
        1,
        Some(session_creator),
        false,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_auto_advance_authorized_role_loop_until_task_session_binding(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    project_root: &str,
    workflow_id: &str,
    actor_id: &str,
    max_nodes: usize,
    approved_planned_tasks: Option<&[ProjectDirectorPlannedTask]>,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    run_auto_advance_authorized_role_loop_with_timeout_budget(
        path,
        index,
        readback_db_path,
        runner,
        director,
        project_root,
        workflow_id,
        actor_id,
        max_nodes,
        approved_planned_tasks,
        1,
        None,
        true,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_auto_advance_authorized_role_loop_with_task_session_bindings(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    project_root: &str,
    workflow_id: &str,
    actor_id: &str,
    max_nodes: usize,
    approved_planned_tasks: &[ProjectDirectorPlannedTask],
    session_creator: &dyn JiaobanNewSessionCreator,
    task_session_bindings: &std::collections::BTreeMap<String, ProjectDirectorTaskSessionBinding>,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    run_auto_advance_authorized_role_loop_with_timeout_budget(
        path,
        index,
        readback_db_path,
        runner,
        director,
        project_root,
        workflow_id,
        actor_id,
        max_nodes,
        Some(approved_planned_tasks),
        1,
        Some(session_creator),
        false,
        Some(task_session_bindings),
    )
}

#[allow(clippy::too_many_arguments)]
fn run_auto_advance_authorized_role_loop_with_timeout_budget(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    project_root: &str,
    workflow_id: &str,
    actor_id: &str,
    max_nodes: usize,
    // 2.2 所批即所跑：Some=合流带进的「用户批过的图」→ 跳过 director.plan 原样执行；None=现状（批后 LM 拆）。
    approved_planned_tasks: Option<&[ProjectDirectorPlannedTask]>,
    // 债二·超时自动重拆剩余预算（壳传 1·递归传 0 → 两连超时只重拆一次·不许无人值守循环）。
    timeout_auto_replan_budget: usize,
    // C1·mode-aware（canon 2026-07-09）：Some=新对话/自动免管路 → 每任务先生后绑建专属会话；
    // None=拐杖（手动挡/existing/旧测试）→ 沿用节点预绑 resume。守卫（path-lock/授权/拒绝）在会话创建之前
    // 就拦，故 None/Some 都不误伤 PanicCreator 守卫。
    session_creator: Option<&dyn JiaobanNewSessionCreator>,
    // 首次用户确认后，主管刚拆完任务即停在绑定面板；复用 needs_binding，不新造阶段。
    pause_for_task_session_binding: bool,
    // Some = 绑定面板确认的逐任务映射；None = 既有自动/重拆路径。
    task_session_bindings: Option<&std::collections::BTreeMap<String, ProjectDirectorTaskSessionBinding>>,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    // 死线·圈固定测试项目（决策 2026-06-27）：非测试 root 入口直接拒——在 LM 拆 / prepare 之前提前拦
    // （纵深防御，不只靠链入口的晚拦）。与现成命令同款 path-lock，闸 / 沙箱本体不动。
    require_test_project_path_lock(project_root, "auto_advance_authorized_role_loop")?;
    let timestamp_ms = unix_timestamp_ms();
    let pid = project_id(project_root);
    // 1. 查 active 方案授权（人闸不省·不创建不跳过；查不到即拒）。
    let store = plan_authorization_store::load_store(path, timestamp_ms)?;
    let active = store
        .authorizations
        .iter()
        .rev()
        .find(|authorization| {
            authorization.project_id == pid
                && authorization.workflow_id == workflow_id
                && authorization.status == PlanAuthorizationStatus::Active
                && authorization
                    .expires_at_ms
                    .is_none_or(|expires_at_ms| expires_at_ms > timestamp_ms)
        })
        .ok_or_else(|| {
            "无 active 方案授权：请先确认方案 + 全局边界复核（自动推进不创建、不跳过授权）。"
                .to_string()
        })?;
    let proposal_id = active
        .source_proposal_id
        .clone()
        .ok_or_else(|| "active 授权缺 source_proposal_id；无法自动推进。".to_string())?;
    let authorization_id = active.authorization_id.clone();
    let auth_revision = store.revision;
    // 质量债·redo 幂等：授权时间窗起点（created_at 之后的口供/链审计都算「本单已完成」——多轮叠加全覆盖）。
    let auth_created_at_ms = active.created_at_ms;
    // 写根为空是合法只读授权：沿用同一人闸和授权记录，不授予写入范围。
    let read_only_authorization = active.scope.allowed_write_roots.is_empty();
    append_role_loop_auto_advance_audit(
        path,
        workflow_id,
        actor_id,
        "role_loop_auto_advance_started",
        if read_only_authorization {
            "已查到 active 只读授权，开始只读自动推进：拆任务 → prepare →（没绑/越界则停）→ 链跑。"
        } else {
            "已查到 active 方案授权，开始授权范围内自动推进：拆任务 → prepare →（没绑/越界则停）→ 链跑。"
        },
    )?;
    // fix3 2.2：从这里（拆任务起）往后**任何失败**都先 append 一条 stopped 审计（阶段+人话·永久留档，
    // 别只活在前端内存/重开 app 就没）再返回 Err——用 IIFE 兜住所有 `?` 与早返回点，一处捕获、绝不漏。
    // 只 append、不改任何既有状态；失败仍失败（Err 语义不变·不吞错）。早期 path-lock/无 active 授权在此之前、不记。
    let advance_result: Result<AutoAdvanceRoleLoopOutcome, String> = (|| {
        // 2. 拿到 planned_tasks —— 两条路：
        //    (a) 2.2 所批即所跑：合流带进「用户批过的图」→ **跳过 director.plan**（不重跑 LM·消除重拆不确定性）；
        //        一致性校验（workflow_id/project 一致·空数组拒），越界仍由下游 prepare guard 逐个钳/拒（此处不放行）。
        //    (b) 现状：加载 ctx + active 授权对应的已确认方案 → director.plan（真主管 LM），2.4 偶发早退自动重试一次。
        // outcome.warnings 累加器：fix3 角色钳位（只 None 路）+ fix4 残料接管（两路）+ fix4 旧链标结（只 re-plan）。
        let mut advance_warnings: Vec<String> = Vec::new();
        let planned_tasks = match approved_planned_tasks {
            Some(approved) => {
                validate_approved_planned_tasks(approved, workflow_id, &pid)?;
                append_role_loop_auto_advance_audit(
                path,
                workflow_id,
                actor_id,
                "role_loop_used_approved_plan_graph",
                &format!(
                    "所批即所跑：采用用户批过的 {} 个任务图，跳过主管重拆（原样进 prepare·guard 仍逐个复核）。",
                    approved.len()
                ),
            )?;
                approved.to_vec()
            }
            None => {
                // 刀B·记忆召回（真实 path·不死锚）：重拆前用手里的 path 填本项目记忆，与预拆/咨询同覆盖。
                let mut ctx = load_project_context(project_root)?;
                ctx.memory_summary = recall_project_memory_summary_at(path, project_root);
                // 质量债·redo 幂等（只 re-plan 分支·调用方真 path 填·死锚纪律照刀B）：授权窗内
                // 已完成事实喂重拆——「这些干完了，别重做」（2026-07-06 双删案根治）。失败 None 不挡重拆。
                ctx.prior_completed_summary =
                    collect_prior_completed_summary(path, workflow_id, auth_created_at_ms);
                let proposal_store =
                    project_consultation_proposal_store::load_store(path, timestamp_ms)?;
                let proposal = proposal_store
                    .proposals
                    .iter()
                    .find(|proposal| proposal.proposal_id == proposal_id)
                    .ok_or_else(|| format!("找不到 active 授权对应的已确认方案：{proposal_id}"))?;
                let (mut tasks, retried) =
                    director_plan_with_retry(director, &ctx, proposal, false)?;
                if retried {
                    append_role_loop_auto_advance_audit(
                        path,
                        workflow_id,
                        actor_id,
                        "role_loop_director_plan_retried",
                        "主管拆任务偶发早退（consult 无输出），已自动重试一次。",
                    )?;
                }
                // fix3 2.1：把 LM 编的界外角色归一到 codex-dev（只收不放）+ 出人话警告。
                advance_warnings.extend(clamp_planned_task_roles(
                    &mut tasks,
                    &proposal.scope_draft.allowed_role_ids,
                ));
                tasks
            }
        };
        // 开工前绑定面板：主管已把任务拆清，但还没有进入 prepare/派发。沿用 `needs_binding` 阶段，
        // 用加法标记让交办主界面显示逐任务映射；这里不写 binding、不建会话、不跑 worker。
        if pause_for_task_session_binding {
            let message = "任务已经拆好。请逐项确认要用的新会话或已有对话，再开始跑。".to_string();
            append_role_loop_auto_advance_audit(
                path,
                workflow_id,
                actor_id,
                "role_loop_auto_advance_stopped",
                &format!("自动推进停在 needs_binding：{message}"),
            )?;
            return Ok(AutoAdvanceRoleLoopOutcome {
                stage: "needs_binding".to_string(),
                planned_task_count: planned_tasks.len(),
                prepared_count: 0,
                needs_binding_count: planned_tasks.len(),
                blocked_count: 0,
                message,
                chain_outcome: None,
                stop_reason: Some("needs_binding".to_string()),
                task_session_binding_required: true,
                planned_tasks,
                warnings: advance_warnings,
            });
        }
        // fix4 2.1：prepare **之前**接管本轮遗留工作项（re-plan 与 approved 两路都做·合法复位·离开 C4 保护状态）。
        advance_warnings.extend(reconcile_stale_work_items_for_plan(
            path,
            project_root,
            &planned_tasks,
        ));
        // fix5 2.2：prepare 前把本轮节点上**超龄**的 running 派发墓碑标结（解除 S1 duplicate_blocked）；未超龄不碰。
        advance_warnings.extend(reconcile_stale_running_dispatches(
            path,
            &planned_tasks,
            unix_timestamp_ms(),
        ));
        // 3. prepare（→ prepared dispatches·授权范围内·把 LM 拆的任务接进派发机器）。
        let prepare_input = PrepareAuthorizedAutoDispatchInput {
            project_root: project_root.to_string(),
            project_id: pid.clone(),
            workflow_id: workflow_id.to_string(),
            proposal_id: proposal_id.clone(),
            authorization_id: authorization_id.clone(),
            actor_id: actor_id.to_string(),
            planned_tasks,
            expected_workflow_revision: None,
            expected_authorization_revision: Some(auth_revision),
            // C1·mode 信号（一处赋值·别造第二套判断）：Some(creator)=C1 自动路=链会每任务绑 →
            // prepare 产 prepared·thread 延迟；None=手动挡/旧壳=现状 needs_binding 判定不变。
            chain_binds_per_task: session_creator.is_some(),
        };
        let prepared = prepare_authorized_auto_dispatch_for_index_at(path, index, &prepare_input)?;
        let planned_task_count = prepared.plan.planned_task_count;
        let prepared_count = prepared.plan.prepared_dispatch_count;
        let needs_binding_count = prepared.plan.needs_binding_count;
        let blocked_count = prepared.plan.blocked_count;
        // 绑定面板的已有会话在 prepare 物化每任务 work_item 后精确落到该任务 node + work_item；
        // 新会话仍留给链内 C1。任何绑定失败都外抛停下，绝不回落到旧 role 节点绑定。
        if let Some(bindings) = task_session_bindings {
            bind_existing_task_sessions(
                path,
                index,
                project_root,
                &prepared.plan.planned_tasks,
                bindings,
            )?;
        }
        // 4. 件 C-1 分流：没 prepared 就停（越界/没绑/无可派）——可见、等用户、不自动绑、不重试。
        if prepared_count == 0 {
            // 收集具体停因（方案缺了什么·给用户可操作反馈，别只笼统说"越界"）：汇被阻断任务的 blocked_reasons。
            let reasons: Vec<String> = prepared
                .plan
                .planned_tasks
                .iter()
                .flat_map(|task| task.blocked_reasons.iter().cloned())
                .collect::<std::collections::BTreeSet<String>>()
                .into_iter()
                .collect();
            let reasons_text = if reasons.is_empty() {
                String::new()
            } else {
                format!("（具体：{}）", reasons.join("；"))
            };
            let (stage, message) = if blocked_count > 0 {
                (
                    "blocked",
                    if read_only_authorization {
                        format!(
                            "有只读任务超出已确认的读取、角色或工具范围被阻断{reasons_text}——本单不授予写入；请按停因调整方案或等待你的决定。"
                        )
                    } else {
                        format!(
                            "有任务超出方案授权范围被阻断{reasons_text}——这单的授权没带可执行范围（多半是方案被判成了纯建议）。请点[重新出方案]把要动手的内容说清楚——写范围由系统自动装配，不需要你手填。"
                        )
                    },
                )
            } else if needs_binding_count > 0 {
                (
                    "needs_binding",
                    "需先给 codex-dev 节点绑一条 Codex 会话再自动推进（本命令不自动绑会话）。"
                        .to_string(),
                )
            } else {
                (
                    "no_dispatchable",
                    "没有可派发的 prepared 任务；停。".to_string(),
                )
            };
            append_role_loop_auto_advance_audit(
                path,
                workflow_id,
                actor_id,
                "role_loop_auto_advance_stopped",
                &format!("自动推进停在 {stage}：{message}"),
            )?;
            return Ok(AutoAdvanceRoleLoopOutcome {
                stage: stage.to_string(),
                planned_task_count,
                prepared_count,
                needs_binding_count,
                blocked_count,
                message,
                chain_outcome: None,
                stop_reason: Some(stage.to_string()),
                task_session_binding_required: false,
                planned_tasks: prepared.plan.planned_tasks.clone(),
                warnings: advance_warnings.clone(),
            });
        }
        // fix4 2.2：**只 re-plan（None）路径**——起链前把该 workflow 未收尾的旧链记录正式标结（superseded），
        // 之后 run_director_task_chain 里的 ensure 走「新建记录」分支、fix3 的「已接续」不再触发（re-plan=从头重来）。
        // approved（Some·所批即所跑）/ C1 的**续跑语义不动**（不 finalize·仍走既有断点续）。
        if approved_planned_tasks.is_none() {
            if let Some(message) = finalize_stale_chain_for_replan(path, project_root, workflow_id)?
            {
                push_unique(&mut advance_warnings, &message);
            }
        }
        // 5. prepared 出来 → **起链前复查授权仍 active**（2.5·批与拆/prepare 之间 LM 耗时长·可能被撤/过期）→ 跑 worker 链
        //    （四护栏·入口 path-lock 圈测试项目·失败即停）。
        require_active_authorization(path, project_root, workflow_id)?;
        // C1·mode-aware：新对话/自动路（Some）→ 复用首轮 run_director_task_chain_with_session_creator 每任务先生后绑；
        // 拐杖路（None·手动挡/existing）→ 沿用节点预绑 resume。**别造第二套**·失败即停在 chain 里已立（不回落）。
        let outcome = match (session_creator, task_session_bindings) {
            (Some(creator), Some(bindings)) => run_director_task_chain_with_task_session_bindings(
                path,
                index,
                readback_db_path,
                runner,
                project_root,
                workflow_id,
                &prepared.plan.planned_tasks,
                max_nodes,
                creator,
                bindings,
            )?,
            (Some(creator), None) => run_director_task_chain_with_session_creator(
                path,
                index,
                readback_db_path,
                runner,
                project_root,
                workflow_id,
                &prepared.plan.planned_tasks,
                max_nodes,
                creator,
            )?,
            (None, None) => run_director_task_chain(
                path,
                index,
                readback_db_path,
                runner,
                project_root,
                workflow_id,
                &prepared.plan.planned_tasks,
                max_nodes,
            )?,
            (None, Some(_)) => {
                return Err("逐任务会话映射缺少 C1 建会话入口，不能派发。".to_string())
            }
        };
        let stop_reason = outcome.stopped_reason.clone();
        let message = format!(
            "授权后自动推进跑完 worker 链：completed {} / dispatched {}{}",
            outcome.completed,
            outcome.dispatched,
            stop_reason
                .as_deref()
                .map(|reason| format!("；停因 {reason}"))
                .unwrap_or_else(|| "；全跑完".to_string())
        );
        append_role_loop_auto_advance_audit(
            path,
            workflow_id,
            actor_id,
            "role_loop_auto_advance_ran",
            &message,
        )?;
        Ok(AutoAdvanceRoleLoopOutcome {
            stage: "ran".to_string(),
            planned_task_count,
            prepared_count,
            needs_binding_count,
            blocked_count,
            message,
            chain_outcome: Some(outcome),
            stop_reason,
            task_session_binding_required: false,
            planned_tasks: prepared.plan.planned_tasks.clone(),
            warnings: advance_warnings,
        })
    })();
    // fix3 2.2：拆/prepare/起链的任何 Err（含 retry 后仍败）→ 留档再返回。stage=blocked/needs_binding 走的是
    // Ok（内层已各记 stopped），不进这里；这里只补「确认后失败但没记」的那批（今晚审计空白的根因）。
    if let Err(error) = &advance_result {
        let _ = append_role_loop_auto_advance_audit(
            path,
            workflow_id,
            actor_id,
            "role_loop_auto_advance_stopped",
            &format!("自动推进失败（已留档）：{error}"),
        );
    }
    // ===== 质量债·超时反馈边：任务超时导致链 fail-stop → 自动打回主管重拆**一次** =====
    // 信任级 = fix3 [接着跑,不用重批]（已确认方案+active 授权下重拆不需重批），只是省了那下人肉点击；
    // **只给 timeout**（供给类/gate 拒/普通 failed 不走——额度死自动重拆=白烧、gate 拒=该人看）；
    // 递归走**现成** re-plan 路（approved=None）——授权复查（1253 双点）/fix9 守卫/path-lock/prepare guard/
    // 四护栏全套照过，不复制路径；预算递减 → 新一轮再 fail-stop（含再超时）预算=0 直接回到人（永不冻）。
    if timeout_auto_replan_budget > 0 {
        if let Ok(outcome) = &advance_result {
            if let Some(chain) = &outcome.chain_outcome {
                if let Some(timed_out_title) = timeout_auto_replan_decision(path, chain) {
                    {
                        let _ = append_role_loop_auto_advance_audit(
                            path,
                            workflow_id,
                            actor_id,
                            "role_loop_timeout_auto_replan",
                            &format!(
                                "任务「{timed_out_title}」超时，自动打回主管重拆（1/1·重拆带已完成事实·授权复查与全套闸照过）。"
                            ),
                        );
                        return match run_auto_advance_authorized_role_loop_with_timeout_budget(
                            path,
                            index,
                            readback_db_path,
                            runner,
                            director,
                            project_root,
                            workflow_id,
                            actor_id,
                            max_nodes,
                            None, // 重拆 = re-plan 路（自然带上已完成事实 + 超时事实行）
                            timeout_auto_replan_budget - 1,
                            session_creator, // C1·重拆轮同 mode（Some 则每任务仍新会话·透传）
                            false, // 重拆不再弹绑定面板。
                            None,  // 新任务不继承上一轮任务→会话映射，一律由 C1 新建。
                        ) {
                            Ok(mut second) => {
                                second.warnings.insert(
                                    0,
                                    format!(
                                        "任务「{timed_out_title}」上轮超时，已自动打回主管重拆 1 次。"
                                    ),
                                );
                                // 新一轮又没跑完（任何原因）→ 人话前缀说明预算已用，回到人。
                                if second
                                    .chain_outcome
                                    .as_ref()
                                    .and_then(|chain| chain.stopped_reason.as_ref())
                                    .is_some()
                                    || second.stage != "ran"
                                {
                                    second.message =
                                        format!("已自动重拆过 1 次：{}", second.message);
                                }
                                Ok(second)
                            }
                            Err(error) => Err(format!("已自动重拆过 1 次，仍失败：{error}")),
                        };
                    }
                }
            }
        }
    }
    advance_result
}

#[tauri::command]
async fn auto_advance_authorized_role_loop(
    request: AutoAdvanceAuthorizedRoleLoopRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    // index/path 在 await 前从 state 取（State 不能跨进 'static 闭包）——同 start_project_director_chain 范本。
    let path = state.workflow_state_path.clone();
    let index = read_index(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let readback_db_path = codex_db::default_state_db_path();
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let director = CliDirectorAgent::default();
        let actor_id = request
            .actor_id
            .clone()
            .unwrap_or_else(|| "role-loop-auto-advance".to_string());
        // C1·独立[接着跑]=天然新对话免管路（无 session_choice·canon 2026-07-09）→ 每任务先生后绑建专属会话。
        run_auto_advance_authorized_role_loop_with_session_creator(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &director,
            &request.project_root,
            &request.workflow_id,
            &actor_id,
            request.max_nodes.unwrap_or(50),
            // 独立自动推进命令：无「已批图」入口 → 现状 LM 拆（2.2 只在合流命令收图）。
            None,
            &ManualRelayJiaobanNewSessionCreator,
        )
    })
    .await
    .map_err(|error| format!("自动推进执行线程异常：{error}"))?
}

// ===== 交办地基 2.2·合流命令：用户点[允许并开始] → 一口气 确认方案→边界复核→授权生效→(绑会话)→自动推进 =====
// **人闸不省**：本命令 = 用户刚点[允许]的直接效果——step1 校验方案 = PendingUserConfirmation、本次调用记录用户确认；
// **无任何免用户路径**（不给定时器/链/别的命令调用口）。步骤全复用现成状态机（record_decision / boundary review /
// auto_advance 内层），不旁路。session_choice=new = 方案a「先生后绑」（2026-07-05 决策
// decisions/2026-07-05-jiaoban-new-session-birth-before-bind-v1.md）：先经现成 manual_relay GUI 直发
// new_session 单次路径在固定测试项目真建一条会话（初始化消息）→ 回执取 thread_id → 走 existing 同款绑定；
// 链照旧 resume（execute 的 resume-only 不反转·commands/runner/relay 本体只调不改）。

// 方案a·新会话出生口（可注入·单测 stub）：成功返回新会话 thread_id；失败返回人话原因。
// 失败由调用方走 fix3 留档（stopped 审计），**不静默回落 existing**。
pub(crate) trait JiaobanNewSessionCreator {
    fn create_initialized_session(
        &self,
        initialization_text: &str,
        requested_by: &str,
    ) -> Result<String, String>;
}

// 初始化等待预算：与 worker 单任务预算（600s）同级封顶；正常初始化 ~15-60s（决策已认）。
const JIAOBAN_NEW_SESSION_TIMEOUT_MS: u128 = 600_000;
const JIAOBAN_NEW_SESSION_POLL_INTERVAL_MS: u64 = 1_000;
// 出生→可绑的兜底等待预算：主因（exec 会话被列表显示过滤挡住）已在 find_thread_by_id 修掉；
// 这里只兜「codex 进程退出后 thread 行落它自家 sqlite 晚一拍」的窗口——成功即 break、零成本。
const JIAOBAN_NEW_SESSION_BIND_VISIBILITY_BUDGET_MS: u128 = 30_000;

// 真实现：调 relay 现成单次路径（spawn 返回 running 回执）→ 轮询到终态 → 回执取 thread_id。
// relay 内部闸原样生效（guard 拒/查重拒 → 人话转述，不绕不伪造）；cwd **写死固定测试项目**（不可参数化）。
pub(crate) struct ManualRelayJiaobanNewSessionCreator;

impl JiaobanNewSessionCreator for ManualRelayJiaobanNewSessionCreator {
    fn create_initialized_session(
        &self,
        initialization_text: &str,
        requested_by: &str,
    ) -> Result<String, String> {
        let input = manual_relay::ManualRelayGuiDirectNewSessionInput {
            original_user_text: initialization_text.to_string(),
            target_project_root: WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
            target_cwd: WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
            sandbox: "workspace-write".to_string(),
            allowed_write_roots: vec![WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string()],
            requested_by: requested_by.to_string(),
        };
        let mut receipt = manual_relay::run_manual_relay_gui_direct_new_session_once(
            input,
            &unix_timestamp_string(),
        )
        .map_err(|error| format!("relay 拒了/起不来（{error}）"))?;
        let relay_attempt_id = receipt.relay_attempt_id.clone();
        let started = std::time::Instant::now();
        while receipt.status == "running" {
            if started.elapsed().as_millis() >= JIAOBAN_NEW_SESSION_TIMEOUT_MS {
                let stop_note = match manual_relay::stop_manual_relay_attempt(
                    manual_relay::ManualRelayStopInput {
                        relay_attempt_id: relay_attempt_id.clone(),
                        requested_by: requested_by.to_string(),
                    },
                    &unix_timestamp_string(),
                ) {
                    Ok(_) => "初始化进程已停掉",
                    Err(_) => "且停进程失败（可能残留，可到中转页手动停）",
                };
                return Err(format!(
                    "初始化超时（超过 {} 秒还没跑完），{stop_note}",
                    JIAOBAN_NEW_SESSION_TIMEOUT_MS / 1000
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(
                JIAOBAN_NEW_SESSION_POLL_INTERVAL_MS,
            ));
            receipt = manual_relay::poll_manual_relay_attempt(
                manual_relay::ManualRelayPollInput {
                    relay_attempt_id: relay_attempt_id.clone(),
                    requested_by: requested_by.to_string(),
                },
                &unix_timestamp_string(),
            )
            .map_err(|error| format!("初始化进程查不到状态了（{error}）"))?;
        }
        if receipt.status != "completed_real_codex" {
            let stderr_note = receipt
                .thread_event_summary
                .stderr_summary
                .as_deref()
                .map(|s| format!("；stderr：{s}"))
                .unwrap_or_default();
            return Err(format!(
                "初始化没跑完（状态 {}，exit {:?}）{stderr_note}",
                receipt.status, receipt.exit_code
            ));
        }
        receipt
            .thread_event_summary
            .thread_id
            .clone()
            .filter(|thread_id| !thread_id.trim().is_empty())
            .ok_or_else(|| "初始化跑完了但回执里没有 thread_id（拿不到新会话号）".to_string())
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct ConfirmAndStartAuthorizedRunRequest {
    pub(crate) project_root: String,
    pub(crate) proposal_id: String,
    pub(crate) session_choice: String, // 仅预填绑定面板第一项（"existing" | "new"）
    #[serde(default)]
    pub(crate) session_id: Option<String>, // 顶层选择的预填会话；不在此直接绑定
    #[serde(default)]
    pub(crate) actor_id: Option<String>,
    #[serde(default)]
    pub(crate) max_nodes: Option<usize>,
    // 2.2 所批即所跑：前端回传「用户批过的那份图」（预拆→审→回传）。Some=原样跑不重拆；None/缺=批后 LM 拆（现状）。
    #[serde(default)]
    pub(crate) approved_planned_tasks: Option<Vec<ProjectDirectorPlannedTask>>,
}

// 绑定面板的「开始跑」不是第二道审批：它只接拆好的同一份任务和逐任务会话选择，复查既有 active 授权后继续。
#[derive(serde::Deserialize)]
pub(crate) struct ConfirmProjectDirectorTaskSessionBindingsRequest {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) planned_tasks: Vec<ProjectDirectorPlannedTask>,
    pub(crate) task_session_bindings: Vec<ProjectDirectorTaskSessionBinding>,
    #[serde(default)]
    pub(crate) actor_id: Option<String>,
    #[serde(default)]
    pub(crate) max_nodes: Option<usize>,
}

// 内层（同步·spawn_blocking 里调；可单测·stub 咨询/主管/链/新会话出生口）。
fn run_confirm_and_start_authorized_run_inner(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    _session_creator: &dyn JiaobanNewSessionCreator,
    request: &ConfirmAndStartAuthorizedRunRequest,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    // 死线·圈固定测试项目（与 auto_advance 同款·非测试 root 提前拒）。
    require_test_project_path_lock(&request.project_root, "confirm_and_start_authorized_run")?;
    let actor_id = request
        .actor_id
        .clone()
        .unwrap_or_else(|| "user".to_string());
    let timestamp_ms = unix_timestamp_ms();
    // 1. 载方案 + 校**人闸**：必须 PendingUserConfirmation（本命令=用户刚点允许·不接其它状态·不创建方案）。
    let proposal_store = project_consultation_proposal_store::load_store(path, timestamp_ms)?;
    let proposal = proposal_store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == request.proposal_id)
        .ok_or_else(|| format!("找不到方案：{}", request.proposal_id))?;
    if proposal.status != ProjectConsultationProposalStatus::PendingUserConfirmation {
        return Err(format!(
            "方案不是「待用户确认」状态（当前 {:?}）；本命令只表达「用户刚点允许」、不接其它状态。",
            proposal.status
        ));
    }
    let workflow_id = proposal.workflow_id.clone();
    let proposal_store_revision = proposal_store.revision;
    // 写根空不再等同于空转：它是只读单，仍须经过同一确认与全局边界复核，授权中不授予写入范围。
    let read_only_authorization = proposal.scope_draft.allowed_write_roots.is_empty();
    // 2. 记录用户确认（现成 record_decision·Confirm·actor=用户）→ 建授权。
    let confirmed = project_consultation_proposal_store::record_decision(
        path,
        &RecordProjectConsultationProposalDecisionInput {
            project_root: request.project_root.clone(),
            proposal_id: request.proposal_id.clone(),
            actor_id: actor_id.clone(),
            decision: ProjectConsultationProposalDecisionKind::Confirm,
            summary: if read_only_authorization {
                "用户点[允许并开始]：确认只读单（read_only；不授予写入范围）。".to_string()
            } else {
                "用户点[允许并开始]：确认方案。".to_string()
            },
            expected_proposal_store_revision: Some(proposal_store_revision),
            expected_plan_authorization_store_revision: None,
        },
        timestamp_ms,
        &format!("confirm-and-start-confirm:{}", unix_timestamp_nanos()),
        &format!("confirm-and-start-auth:{}", unix_timestamp_nanos()),
        &format!("confirm-and-start-auth-user:{}", unix_timestamp_nanos()),
    )?;
    // fix3 2.2：record_decision（确认）**成功之后**的任何失败（授权提取/边界复核/拆任务）→ 先 append
    // stopped 审计（人话）再返回 Err（治「今晚审计只有 started、之后空白」）。step5 auto_advance 由它自己留档、
    // 不含在此（避免双记）；record_decision 本身失败=确认闸没过、按包不记（在此之前）。只 append、不改状态、不吞错。
    // C1 收官：原 new 分支的「S0 单条会话说明」已随退 S0 移除（每任务新会话说明由链侧机制给）。
    let post_confirm: Result<(), String> = (|| {
        let authorization = confirmed
            .plan_authorization
            .ok_or_else(|| "确认方案未产出授权对象".to_string())?;
        let revision = confirmed
            .plan_authorization_store_revision
            .ok_or_else(|| "确认方案未产出授权 revision".to_string())?;
        // 3. 记录全局边界复核（Phase A 用户演全局主管·actor=用户·approved）→ 授权生效。
        plan_authorization_store::record_global_boundary_review_with_proposal(
            path,
            &RecordGlobalBoundaryReviewInput {
                project_root: request.project_root.clone(),
                project_id: project_id(&request.project_root),
                workflow_id: workflow_id.clone(),
                proposal_id: confirmed.proposal.proposal_id.clone(),
                authorization_id: authorization.authorization_id.clone(),
                actor_id: actor_id.clone(),
                review_status: "approved".to_string(),
                summary: if read_only_authorization {
                    "用户点[允许并开始]：同时作全局边界批准（只读单；无写权限）。".to_string()
                } else {
                    "用户点[允许并开始]：同时作全局边界批准（Phase A·用户演全局主管）。"
                        .to_string()
                },
                checklist: GlobalBoundaryReviewChecklist {
                    architecture_boundary_checked: true,
                    cross_project_impact_checked: true,
                    permission_scope_checked: true,
                    read_write_scope_checked: true,
                    tool_and_check_scope_checked: true,
                    memory_boundary_checked: true,
                    stop_conditions_checked: true,
                    acceptance_criteria_checked: true,
                },
                findings: vec![],
                expected_authorization_revision: Some(revision),
            },
            timestamp_ms + 1,
            &format!("confirm-and-start-boundary:{}", unix_timestamp_nanos()),
        )?;
        if read_only_authorization {
            append_role_loop_auto_advance_audit(
                path,
                &workflow_id,
                &actor_id,
                "role_loop_auto_advance_started",
                "开工口放行只读单：授权写范围为空；任务将以 read-only 沙箱运行，不授予任何写入目录。",
            )?;
        }
        // 4. 顶层会话选择退为绑定面板的前端预填：不再把 existing 直接绑到 codex-dev 单节点。
        //    真正的每任务映射在主管拆完后由新确认命令处理；new 的 C1 路径仍原样留在该命令之后。
        match request.session_choice.as_str() {
            "existing" => {
                // 只保留前端预填值的兼容读取；这里绝不再写 node binding。
                let _top_level_prefill_session_id = request.session_id.as_deref();
            }
            "new" => {}
            other => {
                return Err(format!(
                    "未知 session_choice：{other}（只支持 existing | new）"
                ));
            }
        }
        Ok(())
    })();
    if let Err(error) = &post_confirm {
        let _ = append_role_loop_auto_advance_audit(
            path,
            &workflow_id,
            &actor_id,
            "role_loop_auto_advance_stopped",
            &format!("合流在自动推进前失败（已留档）：{error}"),
        );
    }
    post_confirm?;
    // 5. 主管拆完即停在现成 needs_binding 面：面板确认映射前不 prepare、不建会话、不派发。
    run_auto_advance_authorized_role_loop_until_task_session_binding(
        path,
        index,
        readback_db_path,
        runner,
        director,
        &request.project_root,
        &workflow_id,
        &actor_id,
        request.max_nodes.unwrap_or(50),
        request.approved_planned_tasks.as_deref(),
    )
}

#[tauri::command]
async fn confirm_and_start_authorized_run(
    request: ConfirmAndStartAuthorizedRunRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    // index/path 在 await 前从 state 取；真 codex 长耗时 → spawn_blocking 不冻 UI（同范本）。
    let path = state.workflow_state_path.clone();
    let index = read_index(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let readback_db_path = codex_db::default_state_db_path();
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let director = CliDirectorAgent::default();
        run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &director,
            &ManualRelayJiaobanNewSessionCreator,
            &request,
        )
    })
    .await
    .map_err(|error| format!("合流执行线程异常：{error}"))?
}

fn run_confirm_project_director_task_session_bindings_inner(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    session_creator: &dyn JiaobanNewSessionCreator,
    request: &ConfirmProjectDirectorTaskSessionBindingsRequest,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    require_test_project_path_lock(
        &request.project_root,
        "confirm_project_director_task_session_bindings",
    )?;
    let pid = project_id(&request.project_root);
    validate_approved_planned_tasks(&request.planned_tasks, &request.workflow_id, &pid)?;
    let bindings = validate_task_session_bindings(
        &request.planned_tasks,
        &request.task_session_bindings,
        index,
    )?;
    // 面板不是新审批，但用户在选会话期间授权可能被撤/过期；起链前仍按既有口径拒绝。
    require_active_authorization(path, &request.project_root, &request.workflow_id)?;
    let actor_id = request
        .actor_id
        .clone()
        .unwrap_or_else(|| "user".to_string());
    run_auto_advance_authorized_role_loop_with_task_session_bindings(
        path,
        index,
        readback_db_path,
        runner,
        director,
        &request.project_root,
        &request.workflow_id,
        &actor_id,
        request.max_nodes.unwrap_or(50),
        &request.planned_tasks,
        session_creator,
        &bindings,
    )
}

#[tauri::command]
async fn confirm_project_director_task_session_bindings(
    request: ConfirmProjectDirectorTaskSessionBindingsRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AutoAdvanceRoleLoopOutcome, String> {
    let path = state.workflow_state_path.clone();
    let index = read_index(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let readback_db_path = codex_db::default_state_db_path();
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let director = CliDirectorAgent::default();
        run_confirm_project_director_task_session_bindings_inner(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &director,
            &ManualRelayJiaobanNewSessionCreator,
            &request,
        )
    })
    .await
    .map_err(|error| format!("任务会话绑定确认线程异常：{error}"))?
}

// ===== 交办·刀2 2.1·只读预拆命令：批前看图（pending 方案 → 主管 LM 拆图 → 原样返回·零写盘）=====
// 死线：**零写盘·不 annotate·不 prepare·不碰链**——只 load 方案 + load ctx + director.plan_preview → 返回
// planned_tasks + warnings。执行**唯一入口仍是合流人闸**（预拆产物不直通执行）。path-lock **不加**（与
// run_project_consultation 同款只读咨询形先例·决策 2026-06-25：只读可任意项目·唯一防线=readonly_codex_consult
// 的 read-only 沙箱）。接受 PendingUserConfirmation（批前看图）；已确认方案也能拆（预览无状态·幂等）。
#[derive(serde::Deserialize)]
pub(crate) struct PreviewPendingProposalDirectorPlanRequest {
    pub(crate) project_root: String,
    pub(crate) proposal_id: String,
}

#[derive(serde::Serialize, Debug)]
pub(crate) struct PreviewPendingProposalDirectorPlanOutcome {
    pub(crate) planned_tasks: Vec<ProjectDirectorPlannedTask>,
    pub(crate) warnings: Vec<String>,
}

// 内层（同步·spawn_blocking 里调；可单测·stub director）。
fn run_preview_pending_proposal_director_plan_inner(
    path: &std::path::Path,
    director: &dyn DirectorAgent,
    request: &PreviewPendingProposalDirectorPlanRequest,
) -> Result<PreviewPendingProposalDirectorPlanOutcome, String> {
    let timestamp_ms = unix_timestamp_ms();
    // 1. 取方案（接受任意状态·预览无副作用无执行口）——只读取、不改状态、不记决策。
    let proposal_store = project_consultation_proposal_store::load_store(path, timestamp_ms)?;
    let proposal = proposal_store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == request.proposal_id)
        .ok_or_else(|| format!("找不到方案：{}", request.proposal_id))?;
    // 2. load ctx + 主管 LM 预拆（待确认措辞·2.4 偶发早退自动重试一次）。零写盘。
    // 刀B·记忆召回（真实 path·不死锚）：用手里的 path 填本项目记忆，与咨询/重拆同覆盖。
    let mut ctx = load_project_context(&request.project_root)?;
    ctx.memory_summary = recall_project_memory_summary_at(path, &request.project_root);
    let (mut planned_tasks, retried) = director_plan_with_retry(director, &ctx, proposal, true)?;
    let mut warnings: Vec<String> = Vec::new();
    if retried {
        warnings.push("主管拆任务偶发早退（consult 无输出），已自动重试一次。".to_string());
    }
    // fix3 2.1：预拆给用户看的图必须**已钳后**（所见即所跑）——界外角色归一 codex-dev + 人话警告。
    warnings.extend(clamp_planned_task_roles(
        &mut planned_tasks,
        &proposal.scope_draft.allowed_role_ids,
    ));
    // 3. 悬空依赖 warning（照链的先例·title 不在任务集里的依赖）——预拆只提示、**不建边**（零写盘）。
    let titles: std::collections::BTreeSet<&str> = planned_tasks
        .iter()
        .map(|task| task.title.as_str())
        .collect();
    for task in &planned_tasks {
        for dep in &task.depends_on {
            if !titles.contains(dep.as_str()) {
                warnings.push(format!(
                    "任务「{}」依赖不存在的前置「{dep}」（悬空·真跑时拓扑序不含它）",
                    task.title
                ));
            }
        }
    }
    Ok(PreviewPendingProposalDirectorPlanOutcome {
        planned_tasks,
        warnings,
    })
}

#[tauri::command]
async fn preview_pending_proposal_director_plan(
    request: PreviewPendingProposalDirectorPlanRequest,
    state: tauri::State<'_, AppState>,
) -> Result<PreviewPendingProposalDirectorPlanOutcome, String> {
    // 真 LM 只读 420s 级 → spawn_blocking 不冻 UI（同范本）。
    let path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let director = CliDirectorAgent::default();
        run_preview_pending_proposal_director_plan_inner(&path, &director, &request)
    })
    .await
    .map_err(|error| format!("预拆执行线程异常：{error}"))?
}

// ===== fix9·开工口守卫单测（自包含·照 worker_report/B1 先例不依赖 lib.rs 测试 helper）=====
// 复刻 2026-07-07 16:48/16:55 事故形态：tier-1 不交 execution_scope → 真分流产出写根空纯建议方案。
#[cfg(test)]
mod fix9_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_dir(tag: &str) -> PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("fix9-{tag}-{uniq}"));
        fs::create_dir_all(&dir).expect("tmp dir");
        dir
    }

    fn fixture_project_record(project_root: &str) -> ProjectRecord {
        ProjectRecord {
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

    // 碰到即炸的桩：守卫必须在 LM/链/新会话之前拦住——任何一桩被调都说明守卫漏了。
    struct PanicRunner;
    impl CodexResumeRunner for PanicRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            _prompt: &str,
            _last_message_path: &std::path::Path,
            _options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            panic!("fix9 守卫应在 runner 之前拦住");
        }
    }
    struct PanicCreator;
    impl JiaobanNewSessionCreator for PanicCreator {
        fn create_initialized_session(
            &self,
            _initialization_text: &str,
            _requested_by: &str,
        ) -> Result<String, String> {
            panic!("fix9 守卫应在新会话出生口之前拦住");
        }
    }

    struct OneTaskDirector;
    impl DirectorAgent for OneTaskDirector {
        fn plan(
            &self,
            _ctx: &ProjectContext,
            proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            Ok(vec![ProjectDirectorPlannedTask {
                planned_task_id: format!("planned-task:{}:1", proposal.workflow_id),
                title: "核验 index.html".to_string(),
                task_goal: "核验 index.html".to_string(),
                scope: director_task_scope_from_proposal(proposal, "codex-dev"),
                depends_on: vec![],
                acceptance_criteria: vec!["返回核验结论".to_string()],
                report_format: vec!["做了什么".to_string()],
                status: "planned".to_string(),
                guard_result: None,
                work_item_id: None,
                workflow_node_id: None,
                task_package_id: None,
                memory_packet_snapshot_id: None,
                prepared_dispatch_id: None,
                blocked_reasons: vec![],
            }])
        }
    }

    /// 真分流产出的纯建议方案（execution_scope=None → 写根空）：直接用生产 API 造，
    /// 顺带断言事故形态成立（分流本体 0-diff，这里只是消费它）。
    fn create_advice_only_pending_proposal(
        path: &std::path::Path,
        project_root: &str,
    ) -> ProjectConsultationProposal {
        let consult = ConsultationProposal {
            user_goal: "加回 1 个怪".to_string(),
            goal_summary: "在游戏里加回 1 个怪物".to_string(),
            scope_note: "（tier-1 忘了给 execution_scope 的事故形态）".to_string(),
            reasoning: vec!["复刻 16:48 事故".to_string()],
            risks: vec![],
            must_stop_points: vec![],
            next_steps: vec!["加怪".to_string()],
            execution_scope: None,
            suggest_workflow: false,
        };
        let c1 = map_consultation_to_c1_input(&consult, project_root, "consultant").expect("map");
        assert!(
            c1.scope_draft.allowed_write_roots.is_empty(),
            "前置：None 支应映射成写根空（事故形态成立）"
        );
        project_consultation_proposal_store::create_proposal(
            path,
            &c1,
            unix_timestamp_ms(),
            &format!("fix9-proposal:{}", unix_timestamp_nanos()),
        )
        .expect("create proposal")
        .proposal
    }

    fn read_state_json(path: &std::path::Path) -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(path).expect("read state")).expect("parse state")
    }

    // §4①：纯建议方案（写根空）→ 同一人闸建只读授权，并停在既有逐任务绑定面。
    #[test]
    fn advice_only_confirm_authorizes_readonly_and_waits_for_task_binding() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("confirm-guard");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project_record(test_root))
            .expect("bootstrap");
        let proposal = create_advice_only_pending_proposal(&path, test_root);
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: proposal.proposal_id.clone(),
            session_choice: "existing".to_string(),
            session_id: Some("thread-any".to_string()),
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
        };
        let outcome = run_confirm_and_start_authorized_run_inner(
            &path,
            &serde_json::json!({"projects": [{"project_root": test_root}]}),
            &dir.join("readback.sqlite"),
            &PanicRunner,
            &OneTaskDirector,
            &PanicCreator,
            &request,
        )
        .expect("只读单应建授权并进入逐任务绑定面");
        assert!(
            outcome.task_session_binding_required && outcome.stage == "needs_binding",
            "只读单应和普通单一样等待逐任务绑定：{outcome:?}"
        );
        let auth_store =
            plan_authorization_store::load_store(&path, unix_timestamp_ms()).expect("auth store");
        let active = auth_store
            .authorizations
            .iter()
            .find(|authorization| authorization.status == PlanAuthorizationStatus::Active)
            .expect("只读单应有 active 授权");
        assert!(
            active.scope.allowed_write_roots.is_empty(),
            "只读授权绝不可授予写入范围：{:?}",
            active.scope.allowed_write_roots
        );
        assert!(
            active
                .user_confirmation
                .as_ref()
                .is_some_and(|confirmation| confirmation.confirmation_summary.contains("read_only")),
            "既有确认记录应明确只读语义：{active:?}"
        );
        let state = read_state_json(&path);
        let started_reasons: Vec<String> = state["audit_events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .filter(|event| event["event_type"] == "role_loop_auto_advance_started")
                    .filter_map(|event| event["reason"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            started_reasons
                .iter()
                .any(|reason| reason.contains("开工口放行只读单")),
            "应留既有 started 事件中的只读放行说明：{started_reasons:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4②：空写根 active 授权（历史残留形态）+ [接着跑] → 合法只读推进，仍在绑定面停住。
    #[test]
    fn advice_only_active_authorization_advances_to_task_binding() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("advance-guard");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project_record(test_root))
            .expect("bootstrap");
        let proposal = create_advice_only_pending_proposal(&path, test_root);
        // 复刻 16:48 存量链路：确认 + 边界复核 → 空写根授权 active（当时守卫不存在，垃圾已入库）。
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "复刻事故：确认纯建议方案。".to_string(),
                expected_proposal_store_revision: None,
                expected_plan_authorization_store_revision: None,
            },
            unix_timestamp_ms(),
            &format!("fix9-confirm:{}", unix_timestamp_nanos()),
            &format!("fix9-auth:{}", unix_timestamp_nanos()),
            &format!("fix9-auth-user:{}", unix_timestamp_nanos()),
        )
        .expect("confirm");
        let authorization = confirmed.plan_authorization.expect("授权对象");
        plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &RecordGlobalBoundaryReviewInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: proposal.workflow_id.clone(),
                proposal_id: proposal.proposal_id.clone(),
                authorization_id: authorization.authorization_id.clone(),
                actor_id: "user-fixture".to_string(),
                review_status: "approved".to_string(),
                summary: "复刻事故：边界批准。".to_string(),
                checklist: GlobalBoundaryReviewChecklist {
                    architecture_boundary_checked: true,
                    cross_project_impact_checked: true,
                    permission_scope_checked: true,
                    read_write_scope_checked: true,
                    tool_and_check_scope_checked: true,
                    memory_boundary_checked: true,
                    stop_conditions_checked: true,
                    acceptance_criteria_checked: true,
                },
                findings: vec![],
                expected_authorization_revision: confirmed.plan_authorization_store_revision,
            },
            unix_timestamp_ms(),
            &format!("fix9-boundary:{}", unix_timestamp_nanos()),
        )
        .expect("boundary review → active");
        // 前置：确实存在空写根 active 授权（事故残留形态成立）。
        let auth_store =
            plan_authorization_store::load_store(&path, unix_timestamp_ms()).expect("auth store");
        let active = auth_store
            .authorizations
            .iter()
            .find(|authorization| authorization.status == PlanAuthorizationStatus::Active)
            .expect("应有 active 授权");
        assert!(
            active.scope.allowed_write_roots.is_empty(),
            "前置：active 授权写根应为空（事故形态）"
        );
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &serde_json::json!({"projects": [{"project_root": test_root}]}),
            &dir.join("readback.sqlite"),
            &PanicRunner,
            &OneTaskDirector,
            test_root,
            &proposal.workflow_id,
            "user-fixture",
            10,
            None,
        )
        .expect("空写根 active 授权应作为只读单推进");
        assert!(
            outcome.stage == "needs_binding" && outcome.needs_binding_count == 1,
            "只读任务应通过 prepare 并等待绑定：{outcome:?}"
        );
        let state = read_state_json(&path);
        let task_package = state["artifacts"]
            .as_array()
            .and_then(|artifacts| {
                artifacts.iter().find(|artifact| artifact["artifact_type"] == "task_package")
            })
            .expect("只读任务应物化任务包");
        assert!(
            task_package["allowed_write"]
                .as_array()
                .is_some_and(|allowed_write| allowed_write.is_empty()),
            "只读任务包必须保留空 allowed_write：{task_package:?}"
        );
        assert!(
            state["audit_events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .filter(|event| event["event_type"] == "role_loop_auto_advance_started")
                    .filter_map(|event| event["reason"].as_str())
                    .any(|reason| reason.contains("只读自动推进"))
            })
            .unwrap_or(false),
            "接着跑应记录只读 started 审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4③：正常档位方案（execution_scope Some → 档位写根非空）不触守卫——确认后主管可拆任务，
    // 但必须停在新的逐任务绑定面板，不能因顶层旧会话直接绑定或派发。
    #[test]
    fn fix9_guard_does_not_touch_profile_backed_proposal() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("no-false-positive");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project_record(test_root))
            .expect("bootstrap");
        let consult = ConsultationProposal {
            user_goal: "加回 1 个怪".to_string(),
            goal_summary: "在游戏里加回 1 个怪物".to_string(),
            scope_note: "要改文件".to_string(),
            reasoning: vec!["r".to_string()],
            risks: vec![],
            must_stop_points: vec![],
            next_steps: vec!["改 index.html".to_string()],
            execution_scope: Some(ConsultationExecutionScope {
                write_roots: vec![],
                target_files: vec!["index.html".to_string()],
                tools: vec![],
                checks: vec!["浏览器打开看效果".to_string()],
            }),
            suggest_workflow: false,
        };
        let c1 = map_consultation_to_c1_input(&consult, test_root, "consultant").expect("map");
        assert!(
            !c1.scope_draft.allowed_write_roots.is_empty(),
            "前置：档位方案写根非空"
        );
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1,
            unix_timestamp_ms(),
            &format!("fix9-normal:{}", unix_timestamp_nanos()),
        )
        .expect("create");
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
            session_choice: "existing".to_string(),
            session_id: Some("thread-not-in-index".to_string()),
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
        };
        let outcome = run_confirm_and_start_authorized_run_inner(
            &path,
            &serde_json::json!({"projects": [{"project_root": test_root}]}),
            &dir.join("readback.sqlite"),
            &PanicRunner,
            &OneTaskDirector,
            &PanicCreator,
            &request,
        )
        .expect("正常档位方案应停在逐任务绑定面板");
        assert!(
            outcome.task_session_binding_required && outcome.stage == "needs_binding",
            "正常档位方案不得被纯建议守卫误伤：{outcome:?}"
        );
        assert_eq!(
            outcome.prepared_count, 0,
            "绑定面板前不得因顶层旧会话 prepare 或派发"
        );
        let _ = fs::remove_dir_all(dir);
    }
}

// ===== 质量债·redo 幂等 + 超时反馈边 单测（自包含·照 fix9_tests 先例不依赖 lib.rs helper）=====
#[cfg(test)]
mod quality_debt_tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_dir(tag: &str) -> PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("qdebt-{tag}-{uniq}"));
        fs::create_dir_all(&dir).expect("tmp dir");
        dir
    }

    fn fixture_project_record(project_root: &str) -> ProjectRecord {
        ProjectRecord {
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

    fn fixture_index(project_root: &str, thread_id: &str) -> Value {
        serde_json::json!({
          "projects": [{ "project_root": project_root }],
          "threads": [{
            "thread_id": thread_id,
            "project_root": project_root,
            "title": format!("Session {thread_id}"),
            "rollout_exists": true,
            "rollout_path": format!("/tmp/{thread_id}.jsonl")
          }]
        })
    }

    // 造 active 授权（档位方案·写根非空）+ 绑会话 → auto_advance 可一路跑到链。
    fn seed_active_run(path: &std::path::Path, index: &Value, test_root: &str) -> String {
        bootstrap_project_workflow_at(path, &fixture_project_record(test_root)).expect("bootstrap");
        let consult = ConsultationProposal {
            user_goal: "减少一个怪物".to_string(),
            goal_summary: "把游戏里的怪物减一个".to_string(),
            scope_note: "改文件".to_string(),
            reasoning: vec!["r".to_string()],
            risks: vec![],
            must_stop_points: vec![],
            next_steps: vec!["改 index.html".to_string()],
            execution_scope: Some(ConsultationExecutionScope {
                write_roots: vec![],
                target_files: vec!["index.html".to_string()],
                tools: vec![],
                checks: vec![],
            }),
            suggest_workflow: false,
        };
        let c1 = map_consultation_to_c1_input(&consult, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            path,
            &c1,
            unix_timestamp_ms(),
            &format!("qdebt-proposal:{}", unix_timestamp_nanos()),
        )
        .expect("proposal");
        let confirmed = project_consultation_proposal_store::record_decision(
            path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "质量债测试确认。".to_string(),
                expected_proposal_store_revision: None,
                expected_plan_authorization_store_revision: None,
            },
            unix_timestamp_ms(),
            &format!("qdebt-confirm:{}", unix_timestamp_nanos()),
            &format!("qdebt-auth:{}", unix_timestamp_nanos()),
            &format!("qdebt-auth-user:{}", unix_timestamp_nanos()),
        )
        .expect("confirm");
        let authorization = confirmed.plan_authorization.expect("授权对象");
        plan_authorization_store::record_global_boundary_review_with_proposal(
            path,
            &RecordGlobalBoundaryReviewInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: created.proposal.workflow_id.clone(),
                proposal_id: created.proposal.proposal_id.clone(),
                authorization_id: authorization.authorization_id.clone(),
                actor_id: "user-fixture".to_string(),
                review_status: "approved".to_string(),
                summary: "质量债测试边界批准。".to_string(),
                checklist: GlobalBoundaryReviewChecklist {
                    architecture_boundary_checked: true,
                    cross_project_impact_checked: true,
                    permission_scope_checked: true,
                    read_write_scope_checked: true,
                    tool_and_check_scope_checked: true,
                    memory_boundary_checked: true,
                    stop_conditions_checked: true,
                    acceptance_criteria_checked: true,
                },
                findings: vec![],
                expected_authorization_revision: confirmed.plan_authorization_store_revision,
            },
            unix_timestamp_ms(),
            &format!("qdebt-boundary:{}", unix_timestamp_nanos()),
        )
        .expect("boundary → active");
        let workflow_id = created.proposal.workflow_id.clone();
        // 绑会话（codex-dev 节点·existing 同款机器）——auto_advance 的链要 resume 它。
        bind_workflow_node_codex_session_for_index_at(
            path,
            index,
            &WorkflowNodeSessionBindRequest {
                project_root: test_root.to_string(),
                node_id: format!("{workflow_id}:node:codex-dev"),
                work_item_id: None,
                thread_id: "thread-qdebt".to_string(),
            },
        )
        .expect("bind session");
        workflow_id
    }

    // 拆任务 stub：每次被调记录收到的 ctx.prior_completed_summary；产 2 任务（t2 依赖 t1）。
    struct RecordingDirector {
        calls: Cell<usize>,
        seen_prior: RefCell<Vec<Option<String>>>,
    }
    impl RecordingDirector {
        fn new() -> Self {
            Self {
                calls: Cell::new(0),
                seen_prior: RefCell::new(vec![]),
            }
        }
    }
    impl DirectorAgent for RecordingDirector {
        fn plan(
            &self,
            ctx: &ProjectContext,
            proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            self.calls.set(self.calls.get() + 1);
            self.seen_prior
                .borrow_mut()
                .push(ctx.prior_completed_summary.clone());
            let scope = director_task_scope_from_proposal(proposal, "codex-dev");
            let mk = |id: usize, title: &str, deps: Vec<String>| ProjectDirectorPlannedTask {
                planned_task_id: format!("planned-task:{}:{}", proposal.workflow_id, id),
                title: title.to_string(),
                task_goal: format!("自包含目标：{title}"),
                scope: scope.clone(),
                depends_on: deps,
                acceptance_criteria: vec!["可验收".to_string()],
                report_format: vec!["做了什么".to_string()],
                status: "planned".to_string(),
                guard_result: None,
                work_item_id: None,
                workflow_node_id: None,
                task_package_id: None,
                memory_packet_snapshot_id: None,
                prepared_dispatch_id: None,
                blocked_reasons: vec![],
            };
            Ok(vec![
                mk(1, "删一个怪", vec![]),
                mk(2, "浏览器验收", vec!["删一个怪".to_string()]),
            ])
        }
    }

    struct BombDirector2;
    impl DirectorAgent for BombDirector2 {
        fn plan(
            &self,
            _ctx: &ProjectContext,
            _proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            panic!("approved graph 路径不得调 director");
        }
    }

    // 脚本化 runner：按序弹出行为（"ok"=完成+契约口供 / "blocked"=完成态返回求助口供 /
    // "timeout"=超时被杀 / "fail"=普通失败 / "provider_err"=供给类 Err / 脚本耗尽默认 ok）。
    struct ScriptedRunner {
        script: RefCell<Vec<&'static str>>,
        calls: Cell<usize>,
    }
    impl ScriptedRunner {
        fn new(script: Vec<&'static str>) -> Self {
            Self {
                script: RefCell::new(script),
                calls: Cell::new(0),
            }
        }
    }
    impl CodexResumeRunner for ScriptedRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            _prompt: &str,
            last_message_path: &Path,
            _options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            self.calls.set(self.calls.get() + 1);
            let behavior = {
                let mut script = self.script.borrow_mut();
                if script.is_empty() {
                    "ok"
                } else {
                    script.remove(0)
                }
            };
            if behavior == "provider_err" {
                return Err("codex_provider_unavailable:codex 额度用完了".to_string());
            }
            if let Some(parent) = last_message_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            match behavior {
                "ok" => {
                    let _ = fs::write(
                        last_message_path,
                        "干完了。\n```json\n{\"did\":\"已删除第三个巡逻怪\",\"outputs\":[\"/t/index.html\"],\"status\":\"done\",\"evidence\":[\"看过\"]}\n```",
                    );
                    Ok((
                        CodexResumeRunResult {
                            exit_code: 0,
                            timed_out: false,
                            stderr_summary: None,
                        },
                        WorkflowNodeDispatchExecutionOptions {
                            readback_stats: Some(CodexDispatchReadbackStats {
                                transcript_event_count: 3,
                                transcript_target_hits: 1,
                            }),
                        },
                    ))
                }
                "blocked" => {
                    let _ = fs::write(
                        last_message_path,
                        "我需要主管处理。\n```json\n{\"did\":\"缺权限无法继续\",\"outputs\":[],\"status\":\"blocked\",\"evidence\":[\"读 /secure 被拒\"],\"permission_requests\":[\"请授权读取 /secure\"],\"open_issues\":[\"缺少真实配置文件\"],\"direction_risks\":[\"继续猜会误改沙箱\"],\"follow_up_suggestions\":[\"主管补充路径后重派\"]}\n```",
                    );
                    Ok((
                        CodexResumeRunResult {
                            exit_code: 0,
                            timed_out: false,
                            stderr_summary: None,
                        },
                        WorkflowNodeDispatchExecutionOptions {
                            readback_stats: Some(CodexDispatchReadbackStats {
                                transcript_event_count: 3,
                                transcript_target_hits: 1,
                            }),
                        },
                    ))
                }
                "timeout" => Ok((
                    CodexResumeRunResult {
                        exit_code: 124,
                        timed_out: true,
                        stderr_summary: Some("killed after timeout".to_string()),
                    },
                    WorkflowNodeDispatchExecutionOptions {
                        readback_stats: None,
                    },
                )),
                _ => Ok((
                    CodexResumeRunResult {
                        exit_code: 1,
                        timed_out: false,
                        stderr_summary: Some("boom".to_string()),
                    },
                    WorkflowNodeDispatchExecutionOptions {
                        readback_stats: None,
                    },
                )),
            }
        }
    }

    fn run_loop(
        path: &std::path::Path,
        index: &Value,
        dir: &std::path::Path,
        runner: &dyn CodexResumeRunner,
        director: &dyn DirectorAgent,
        test_root: &str,
        workflow_id: &str,
        approved: Option<&[ProjectDirectorPlannedTask]>,
    ) -> Result<AutoAdvanceRoleLoopOutcome, String> {
        run_auto_advance_authorized_role_loop(
            path,
            index,
            &dir.join("readback.sqlite"),
            runner,
            director,
            test_root,
            workflow_id,
            "user-fixture",
            10,
            approved,
        )
    }

    fn audit_reasons(path: &std::path::Path, event_type: &str) -> Vec<String> {
        let state: Value =
            serde_json::from_str(&fs::read_to_string(path).expect("state")).expect("json");
        state["audit_events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .filter(|event| event["event_type"] == event_type)
                    .filter_map(|event| event["reason"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    // §4①②·collect 直测：双删案复刻（口供行含 did+status+产物文件名·标题从 work_item）+ 多轮累计
    // （窗内两轮都在·窗外旧口供不进）+ 无自述行 + 超时行 + 0 条 None + 读失败 None。
    #[test]
    fn collect_prior_completed_facts_double_delete_case() {
        let dir = tmp_dir("collect");
        let path = dir.join("workflow-state.v0.json");
        let state = serde_json::json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": "seed",
            "projects": [], "agent_adapters": [], "workflows": [{"workflow_id": "wf-1"}],
            "nodes": [], "edges": [],
            "work_items": [
                {"workflow_id": "wf-1", "work_item_id": "wi-1", "state": "accepted", "title": "删一个怪"},
                {"workflow_id": "wf-1", "work_item_id": "wi-2", "state": "accepted", "title": "第二轮再删"}
            ],
            "artifacts": [], "reviews": [],
            "audit_events": [
                {"event_id": "r1", "event_type": "worker_structured_report_recorded", "workflow_id": "wf-1",
                 "work_item_id": "wi-1", "created_at": "2000",
                 "executed_what": "已删除第三个巡逻怪", "changed_what": "/t/index.html",
                 "acceptance_status": "reported_completed"},
                {"event_id": "r2", "event_type": "worker_structured_report_recorded", "workflow_id": "wf-1",
                 "work_item_id": "wi-2", "created_at": "3000",
                 "executed_what": "又删掉一个（第二轮）", "changed_what": "/t/index.html",
                 "acceptance_status": "reported_completed"},
                {"event_id": "r-old", "event_type": "worker_structured_report_recorded", "workflow_id": "wf-1",
                 "work_item_id": "wi-1", "created_at": "500",
                 "executed_what": "上一单授权的旧口供（不该进）", "changed_what": "x",
                 "acceptance_status": "reported_completed"},
                {"event_id": "c1", "event_type": "workflow_chain_node_completed", "workflow_id": "wf-1",
                 "created_at": "2100", "reason": "薄链驱动：任务「静默完成的任务」真派发成功（dispatch d1）"},
                {"event_id": "f1", "event_type": "workflow_chain_node_failed", "workflow_id": "wf-1",
                 "created_at": "2200", "reason": "薄链驱动：任务「浏览器验收」失败即停——worker 派发未完成（state=failed·timed_out）"}
            ],
            "capabilities": [], "harness_resources": []
        });
        fs::write(&path, serde_json::to_string_pretty(&state).unwrap()).expect("write");
        let summary = collect_prior_completed_summary(&path, "wf-1", 1000).expect("应有事实块");
        assert!(
            summary.contains("「删一个怪」"),
            "标题从 work_item：{summary}"
        );
        assert!(summary.contains("已删除第三个巡逻怪"), "did 在：{summary}");
        assert!(summary.contains("reported_completed"), "status 在");
        assert!(
            summary.contains("/t/index.html"),
            "产物文件名在（不搬内容本体）"
        );
        assert!(
            summary.contains("又删掉一个（第二轮）"),
            "多轮累计：第二轮也在"
        );
        assert!(!summary.contains("旧口供"), "窗外（授权前）不进");
        assert!(
            summary.contains("「静默完成的任务」—（无自述·执行态 completed）"),
            "完成没口供 → 无自述行：{summary}"
        );
        assert!(
            summary.contains("「浏览器验收」— 上轮超时被杀"),
            "超时事实行在：{summary}"
        );
        // 0 条 → None（窗推到未来）。
        assert!(collect_prior_completed_summary(&path, "wf-1", 999_999).is_none());
        // 读失败 → None 不挡重拆。
        assert!(collect_prior_completed_summary(&dir.join("no-such.json"), "wf-1", 0).is_none());
        let _ = fs::remove_dir_all(dir);
    }

    // §4①·prompt 渲染：ctx 填了 → 事实块+禁令在；None → 不渲染（首跑/预拆天然如此）。
    #[test]
    fn prompt_renders_prior_facts_block_only_when_filled() {
        let dir = tmp_dir("prompt");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "thread-qdebt");
        let _wf = seed_active_run(&path, &index, WORKFLOW_ENGINE_TEST_PROJECT_ROOT);
        let store = project_consultation_proposal_store::load_store(&path, unix_timestamp_ms())
            .expect("store");
        let proposal = &store.proposals[0];
        let mut ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        assert!(
            !director_build_prompt(&ctx, proposal).contains("本单已完成"),
            "None → 不渲染"
        );
        ctx.prior_completed_summary =
            Some("「删一个怪」— 已删除第三个巡逻怪（reported_completed）".to_string());
        let prompt = director_build_prompt(&ctx, proposal);
        assert!(prompt.contains("本单已完成"), "块标题在");
        assert!(prompt.contains("别重复执行这些动作"), "禁令在");
        assert!(prompt.contains("已删除第三个巡逻怪"), "事实在");
        let _ = fs::remove_dir_all(dir);
    }

    // 刀B 补渲染回归（质量债线报备逮到的暗债）：memory_summary 填了必须上主管 prompt；None 不渲染。
    #[test]
    fn prompt_renders_memory_summary_when_filled() {
        let dir = tmp_dir("prompt-mem");
        let path = dir.join("workflow-state.v0.json");
        // seed_active_run 的绑定写死 "thread-qdebt"——索引必须给同号（否则「不在索引内」拒绑）。
        let index = fixture_index(WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "thread-qdebt");
        let _wf = seed_active_run(&path, &index, WORKFLOW_ENGINE_TEST_PROJECT_ROOT);
        let store = project_consultation_proposal_store::load_store(&path, unix_timestamp_ms())
            .expect("store");
        let proposal = &store.proposals[0];
        let mut ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        assert!(
            !director_build_prompt(&ctx, proposal).contains("项目记忆"),
            "None → 不渲染"
        );
        ctx.memory_summary =
            Some("[workflow_summary] 游戏怪物已被删到 0——改怪物数先核现状".to_string());
        let prompt = director_build_prompt(&ctx, proposal);
        assert!(prompt.contains("--- 项目记忆"), "块标题在");
        assert!(prompt.contains("游戏怪物已被删到 0"), "记忆内容在");
        assert!(prompt.contains("仍以注入文档为准"), "参考不指令的措辞在");
        let _ = fs::remove_dir_all(dir);
    }

    // §4③+①链级：t1 完成（真口供落库）→ t2 超时 → 自动打回重拆 1 次（审计在）→ 第二轮 ran；
    // 重拆 director 收到的 ctx.prior 含 t1 事实（双删案根治的端到端形）。
    #[test]
    fn timeout_triggers_one_auto_replan_with_facts_then_ran() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("auto-replan");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(test_root, "thread-qdebt");
        let workflow_id = seed_active_run(&path, &index, test_root);
        let director = RecordingDirector::new();
        // 第一轮：t1 ok（落真口供）、t2 timeout；第二轮（重拆后）：全 ok。
        let runner = ScriptedRunner::new(vec!["ok", "timeout"]);
        let outcome = run_loop(
            &path,
            &index,
            &dir,
            &runner,
            &director,
            test_root,
            &workflow_id,
            None,
        )
        .expect("超时应自动重拆后跑完");
        assert_eq!(outcome.stage, "ran", "{outcome:?}");
        assert!(
            outcome
                .warnings
                .iter()
                .any(|w| w.contains("已自动打回主管重拆 1 次")),
            "warnings 记自动重拆：{:?}",
            outcome.warnings
        );
        assert_eq!(director.calls.get(), 2, "初拆 + 超时重拆 = 2 次");
        let seen = director.seen_prior.borrow();
        assert!(seen[0].is_none(), "首拆无前轮事实");
        let second_prior = seen[1].clone().expect("重拆应带已完成事实");
        assert!(
            second_prior.contains("已删除第三个巡逻怪"),
            "重拆看到 t1 口供（双删案根治）：{second_prior}"
        );
        assert!(
            second_prior.contains("上轮超时被杀"),
            "重拆看到超时事实：{second_prior}"
        );
        let replan_audits = audit_reasons(&path, "role_loop_timeout_auto_replan");
        assert_eq!(replan_audits.len(), 1, "自动重拆审计恰一条");
        assert!(replan_audits[0].contains("浏览器验收"), "审计点名超时任务");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn timeout_replan_does_not_inherit_prior_task_session_mapping() {
        struct CountingCreator {
            calls: Cell<usize>,
        }
        impl JiaobanNewSessionCreator for CountingCreator {
            fn create_initialized_session(
                &self,
                _text: &str,
                _by: &str,
            ) -> Result<String, String> {
                self.calls.set(self.calls.get() + 1);
                Ok("thread-qdebt".to_string())
            }
        }

        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("replan-no-binding-inherit");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(test_root, "thread-qdebt");
        let workflow_id = seed_active_run(&path, &index, test_root);
        let proposal_store =
            project_consultation_proposal_store::load_store(&path, unix_timestamp_ms()).expect("proposal store");
        let scope = director_task_scope_from_proposal(&proposal_store.proposals[0], "codex-dev");
        let make_task = |id: usize, title: &str, depends_on: Vec<String>| ProjectDirectorPlannedTask {
            planned_task_id: format!("planned-task:{workflow_id}:{id}"),
            title: title.to_string(),
            task_goal: title.to_string(),
            scope: scope.clone(),
            depends_on,
            acceptance_criteria: vec!["ok".to_string()],
            report_format: vec!["r".to_string()],
            status: "planned".to_string(),
            guard_result: None,
            work_item_id: None,
            workflow_node_id: None,
            task_package_id: None,
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        };
        let approved = vec![
            make_task(1, "删一个怪", vec![]),
            make_task(2, "浏览器验收", vec!["删一个怪".to_string()]),
        ];
        let mappings = std::collections::BTreeMap::from([
            (
                approved[0].planned_task_id.clone(),
                ProjectDirectorTaskSessionBinding {
                    planned_task_id: approved[0].planned_task_id.clone(),
                    session_choice: "existing".to_string(),
                    session_id: Some("thread-qdebt".to_string()),
                },
            ),
            (
                approved[1].planned_task_id.clone(),
                ProjectDirectorTaskSessionBinding {
                    planned_task_id: approved[1].planned_task_id.clone(),
                    session_choice: "new".to_string(),
                    session_id: None,
                },
            ),
        ]);
        let creator = CountingCreator {
            calls: Cell::new(0),
        };
        let runner = ScriptedRunner::new(vec!["ok", "timeout"]);
        let director = RecordingDirector::new();
        let outcome = run_auto_advance_authorized_role_loop_with_task_session_bindings(
            &path,
            &index,
            &dir.join("readback.sqlite"),
            &runner,
            &director,
            test_root,
            &workflow_id,
            "user-fixture",
            10,
            &approved,
            &creator,
            &mappings,
        )
        .expect("超时后应自动重拆一次");
        assert_eq!(outcome.stage, "ran", "{outcome:?}");
        assert_eq!(director.calls.get(), 1, "首轮用已批任务图；只有重拆才调用主管");
        assert_eq!(
            creator.calls.get(),
            3,
            "首轮只有一项选 new；重拆的两项必须都新建，不能继承旧映射"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4④⑤·预算=1 不循环：两连超时只重拆一次，第二轮停·人话含「已自动重拆过 1 次」。
    #[test]
    fn budget_one_no_loop_on_consecutive_timeouts() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("budget");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(test_root, "thread-qdebt");
        let workflow_id = seed_active_run(&path, &index, test_root);
        let director = RecordingDirector::new();
        let runner = ScriptedRunner::new(vec!["timeout", "timeout", "timeout", "timeout"]);
        let outcome = run_loop(
            &path,
            &index,
            &dir,
            &runner,
            &director,
            test_root,
            &workflow_id,
            None,
        )
        .expect("第二轮停但返回 Ok outcome");
        assert_eq!(director.calls.get(), 2, "预算 1：只重拆一次·绝不循环");
        assert!(
            outcome.message.starts_with("已自动重拆过 1 次"),
            "人话前缀：{}",
            outcome.message
        );
        assert!(
            outcome
                .chain_outcome
                .as_ref()
                .and_then(|chain| chain.stopped_reason.as_deref())
                .map(|reason| reason.contains("·timed_out"))
                .unwrap_or(false),
            "第二轮仍超时停·回到人"
        );
        assert_eq!(
            audit_reasons(&path, "role_loop_timeout_auto_replan").len(),
            1,
            "审计恰一条（不循环）"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4⑥·非 timeout 失败不触发：普通 failed / 供给类 Err 都不自动重拆。
    #[test]
    fn non_timeout_failures_do_not_trigger_replan() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        // 普通 failed（state=failed）。
        let dir = tmp_dir("no-trigger-fail");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(test_root, "thread-qdebt");
        let workflow_id = seed_active_run(&path, &index, test_root);
        let director = RecordingDirector::new();
        let runner = ScriptedRunner::new(vec!["fail"]);
        let outcome = run_loop(
            &path,
            &index,
            &dir,
            &runner,
            &director,
            test_root,
            &workflow_id,
            None,
        )
        .expect("失败即停仍 Ok outcome");
        assert_eq!(director.calls.get(), 1, "普通 failed 不重拆");
        assert!(audit_reasons(&path, "role_loop_timeout_auto_replan").is_empty());
        assert!(!outcome.message.contains("已自动重拆"));
        let _ = fs::remove_dir_all(dir);
        // 供给类 Err（execute Err 路·停因不含 ·timed_out 标记）。
        let dir2 = tmp_dir("no-trigger-provider");
        let path2 = dir2.join("workflow-state.v0.json");
        let workflow_id2 = seed_active_run(&path2, &index, test_root);
        let director2 = RecordingDirector::new();
        let runner2 = ScriptedRunner::new(vec!["provider_err"]);
        let _ = run_loop(
            &path2,
            &index,
            &dir2,
            &runner2,
            &director2,
            test_root,
            &workflow_id2,
            None,
        );
        assert_eq!(director2.calls.get(), 1, "供给类不重拆（额度死重拆=白烧）");
        assert!(audit_reasons(&path2, "role_loop_timeout_auto_replan").is_empty());
        let _ = fs::remove_dir_all(dir2);
    }

    // §4⑦·approved graph（所批即所跑）首跑路径两件 0 触碰：director 不被调（Bomb 没炸）、无喂料、
    // 跑通 ran、无重拆审计。
    #[test]
    fn approved_graph_path_untouched() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("approved");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(test_root, "thread-qdebt");
        let workflow_id = seed_active_run(&path, &index, test_root);
        let store = project_consultation_proposal_store::load_store(&path, unix_timestamp_ms())
            .expect("store");
        let proposal = &store.proposals[0];
        let scope = director_task_scope_from_proposal(proposal, "codex-dev");
        let approved = vec![ProjectDirectorPlannedTask {
            planned_task_id: format!("planned-task:{workflow_id}:1"),
            title: "唯一任务".to_string(),
            task_goal: "自包含".to_string(),
            scope,
            depends_on: vec![],
            acceptance_criteria: vec!["ok".to_string()],
            report_format: vec!["r".to_string()],
            status: "planned".to_string(),
            guard_result: None,
            work_item_id: None,
            workflow_node_id: None,
            task_package_id: None,
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        }];
        let runner = ScriptedRunner::new(vec!["ok"]);
        let outcome = run_loop(
            &path,
            &index,
            &dir,
            &runner,
            &BombDirector2,
            test_root,
            &workflow_id,
            Some(&approved),
        )
        .expect("approved 路照旧全通");
        assert_eq!(outcome.stage, "ran");
        assert!(audit_reasons(&path, "role_loop_timeout_auto_replan").is_empty());
        assert!(!outcome.warnings.iter().any(|w| w.contains("自动打回")));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn worker_help_signal_stops_chain_at_waiting_decision_without_completing_task() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("worker-help");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(test_root, "thread-qdebt");
        let workflow_id = seed_active_run(&path, &index, test_root);
        let director = RecordingDirector::new();
        let runner = ScriptedRunner::new(vec!["blocked", "ok"]);
        let outcome = run_loop(
            &path,
            &index,
            &dir,
            &runner,
            &director,
            test_root,
            &workflow_id,
            None,
        )
        .expect("worker 求助应停链等待主管，不应崩");
        let chain = outcome.chain_outcome.as_ref().expect("应有链结果");
        assert_eq!(
            chain.stopped_reason.as_deref(),
            Some("waiting_decision:worker_help:删一个怪")
        );
        assert_eq!(chain.completed, 0, "求助任务不能计 completed");
        assert_eq!(chain.steps.len(), 1, "求助后不继续跑后续任务");
        assert_eq!(chain.steps[0].state, "waiting_decision");
        assert!(chain.steps[0]
            .report_summary
            .as_deref()
            .unwrap_or("")
            .contains("缺权限无法继续"));
        assert_eq!(chain.steps[0].report_status.as_deref(), Some("blocked"));

        let state: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("state")).expect("json");
        let chain_node = state["workflow_chain_runs"][0]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["node_id"].as_str().unwrap_or("").ends_with(":1"))
            .expect("第一任务链节点");
        assert_eq!(chain_node["state"], "waiting_decision");
        assert!(chain_node["message"]
            .as_str()
            .unwrap_or("")
            .contains("worker 求助"));
        let task_node_id = format!("{workflow_id}:node:task:{}", stable_id(&format!("planned-task:{workflow_id}:1")));
        let task_node = state["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["node_id"] == task_node_id)
            .expect("任务级节点");
        assert_eq!(task_node["state"], "waiting_decision");
        let help_audits = audit_reasons(&path, "workflow_chain_node_waiting_decision");
        assert_eq!(help_audits.len(), 1);
        assert!(help_audits[0].contains("worker 求助"));
        assert!(help_audits[0].contains("请授权读取 /secure"));
        let completed_audits = audit_reasons(&path, "workflow_chain_node_completed");
        assert!(
            completed_audits.is_empty(),
            "求助任务不能先写 completed 审计：{completed_audits:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4⑧·stop_requested：用户点过停 → 判定层拒绝自动续（含判别函数本身的三分覆盖）。
    #[test]
    fn stop_requested_blocks_auto_replan_decision() {
        // 判别函数：timeout 形入选；user_stop/普通 fail/供给类不入。
        assert!(chain_timeout_fail_stop_task(Some(
            "fail_stop:node_error:浏览器验收:worker 派发未完成（state=failed·timed_out）"
        ))
        .map(|title| title == "浏览器验收")
        .unwrap_or(false));
        assert!(chain_timeout_fail_stop_task(Some("user_stop_requested")).is_none());
        assert!(chain_timeout_fail_stop_task(Some(
            "fail_stop:node_error:任务:worker 派发未完成（state=failed）"
        ))
        .is_none());
        assert!(chain_timeout_fail_stop_task(Some(
            "fail_stop:node_error:任务:codex_provider_unavailable:额度用完了"
        ))
        .is_none());
        assert!(chain_timeout_fail_stop_task(None).is_none());
        // 判定层：盘上 stop_requested=true → None（点过停不自动续）；false → Some。
        let dir = tmp_dir("stop-req");
        let path = dir.join("workflow-state.v0.json");
        let mk_state = |stop_requested: bool| {
            serde_json::json!({
                "schema_version": "workflow_state_v0", "workflow_version": 1, "updated_at": "seed",
                "projects": [], "agent_adapters": [], "workflows": [{"workflow_id": "wf-1"}],
                "nodes": [], "edges": [], "work_items": [], "artifacts": [], "reviews": [],
                "audit_events": [], "capabilities": [], "harness_resources": [],
                "workflow_chain_runs": [{
                    "chain_run_id": "chain-x", "project_id": "p", "workflow_id": "wf-1",
                    "state": "failed", "stop_requested": stop_requested,
                    "started_at": "1000", "ended_at": "2000", "nodes": []
                }]
            })
        };
        let chain = DirectorChainOutcome {
            total: 1,
            dispatched: 1,
            completed: 0,
            skipped: 0,
            chain_run_id: "chain-x".to_string(),
            steps: vec![],
            director_summary: None,
            warnings: vec![],
            stopped_reason: Some(
                "fail_stop:node_error:浏览器验收:worker 派发未完成（state=failed·timed_out）"
                    .to_string(),
            ),
        };
        fs::write(&path, serde_json::to_string(&mk_state(true)).unwrap()).unwrap();
        assert!(
            timeout_auto_replan_decision(&path, &chain).is_none(),
            "用户点过停 → 不自动续"
        );
        fs::write(&path, serde_json::to_string(&mk_state(false)).unwrap()).unwrap();
        assert_eq!(
            timeout_auto_replan_decision(&path, &chain).as_deref(),
            Some("浏览器验收"),
            "没点停 + timeout 形 → 放行"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // ===== C1·每任务独立会话 单测 =====

    // 造一个 prepared 任务（带 task_package_id·scope 从盘上真方案派生·复用现有夹具）。
    fn c1_fixture_task(
        path: &Path,
        index: &Value,
        artifact_id: Option<&str>,
    ) -> ProjectDirectorPlannedTask {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let workflow_id = seed_active_run(path, index, test_root);
        let store = project_consultation_proposal_store::load_store(path, unix_timestamp_ms())
            .expect("store");
        let scope = director_task_scope_from_proposal(&store.proposals[0], "codex-dev");
        ProjectDirectorPlannedTask {
            planned_task_id: format!("planned-task:{workflow_id}:1"),
            title: "任务甲：删一个巡逻怪".to_string(),
            task_goal: "自包含".to_string(),
            scope,
            depends_on: vec![],
            acceptance_criteria: vec!["ok".to_string()],
            report_format: vec!["r".to_string()],
            status: "prepared".to_string(),
            guard_result: None,
            work_item_id: Some("wi-1".to_string()),
            workflow_node_id: Some("node-1".to_string()),
            task_package_id: artifact_id.map(str::to_string),
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        }
    }

    // §4：target_session_id 物化——命中 artifact 回填 thread；缺 artifact / 无 task_package_id → no-op false（不崩）。
    #[test]
    fn c1_materialize_target_session_id_into_artifact() {
        let dir = tmp_dir("c1-mat");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "thread-qdebt");
        let task = c1_fixture_task(&path, &index, Some("art-c1"));
        let mut value = serde_json::json!({
            "artifacts": [ {"artifact_id": "art-c1", "target_session_id": Value::Null} ]
        });
        assert!(set_task_artifact_target_session_id(
            &mut value,
            &task,
            "thread-xyz"
        ));
        assert_eq!(value["artifacts"][0]["target_session_id"], "thread-xyz");
        // 缺 artifact → false（no-op·不崩）。
        let mut empty = serde_json::json!({ "artifacts": [] });
        assert!(!set_task_artifact_target_session_id(&mut empty, &task, "t"));
        // 无 task_package_id → false（不重新 seed·直接改字段）。
        let mut no_id = task.clone();
        no_id.task_package_id = None;
        assert!(!set_task_artifact_target_session_id(
            &mut value, &no_id, "t2"
        ));
        let _ = fs::remove_dir_all(dir);
    }

    // §4：建会话失败 = fail-loud（返回 Err·供给类人话透传），**绝不静默回落共用会话**（不返回 Ok）。
    #[test]
    fn c1_session_create_failure_is_loud_no_fallback() {
        struct FailingCreator;
        impl JiaobanNewSessionCreator for FailingCreator {
            fn create_initialized_session(&self, _text: &str, _by: &str) -> Result<String, String> {
                Err("codex_provider_unavailable:codex 额度用完了".to_string())
            }
        }
        let dir = tmp_dir("c1-fail");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_index(WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "thread-qdebt");
        let task = c1_fixture_task(&path, &index, Some("art-x"));
        let err = create_and_bind_task_session(
            &path,
            &index,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            "wf-1",
            "node-1",
            "wi-1",
            &task,
            &FailingCreator,
        )
        .expect_err("建会话失败必须 fail-loud 返回 Err，不许静默回落共用会话");
        assert!(err.contains("新建会话失败"), "人话前缀：{err}");
        assert!(err.contains("额度用完"), "供给类人话透传：{err}");
        let _ = fs::remove_dir_all(dir);
    }

    // §4·计次：注入 stub 会话工厂被调 N 次、各任务 thread 互异（工厂本体契约·链级 3× 见回交真跑）。
    #[test]
    fn c1_stub_session_factory_counts_and_distinct_threads() {
        struct CountingCreator {
            calls: Cell<usize>,
        }
        impl JiaobanNewSessionCreator for CountingCreator {
            fn create_initialized_session(&self, text: &str, _by: &str) -> Result<String, String> {
                let n = self.calls.get() + 1;
                self.calls.set(n);
                // 会话以任务命名可辨（智能体页列表）：init 文本里带任务标题。
                assert!(text.contains("任务"), "初始化消息应含任务名：{text}");
                Ok(format!("thread-task-{n}"))
            }
        }
        let creator = CountingCreator {
            calls: Cell::new(0),
        };
        let t1 = creator
            .create_initialized_session("承接任务「甲」", "director_chain")
            .unwrap();
        let t2 = creator
            .create_initialized_session("承接任务「乙」", "director_chain")
            .unwrap();
        let t3 = creator
            .create_initialized_session("承接任务「丙」", "director_chain")
            .unwrap();
        assert_eq!(creator.calls.get(), 3, "每任务一次");
        assert!(t1 != t2 && t2 != t3 && t1 != t3, "各任务 thread 互异");
    }

    // §8·①链级集成测（一次链调用覆盖三断言）：3 任务链经 run_director_task_chain_with_session_creator 真跑
    // （stub 会话工厂 + 记录 runner），直证：creator 被调 3 次 + 各任务 dispatch 用各自新 thread + 3 个
    // target_session_id 互异且物化。prepared-chain 夹具经现成 prepare 造（不手搓状态）。
    #[test]
    fn c1_chain_creates_per_task_session_dispatches_with_it_and_materializes() {
        // 每任务返回互异新 thread（都在 index 里·bind 才认）。
        struct DistinctCreator {
            calls: Cell<usize>,
        }
        impl JiaobanNewSessionCreator for DistinctCreator {
            fn create_initialized_session(&self, _text: &str, _by: &str) -> Result<String, String> {
                let n = self.calls.get() + 1;
                self.calls.set(n);
                Ok(format!("thread-c1-task-{n}"))
            }
        }
        // 记录每次 dispatch 拿到的 thread_id（证「各任务用各自会话」）+ 写口供 → completed。
        struct RecordingRunner {
            threads: RefCell<Vec<String>>,
        }
        impl CodexResumeRunner for RecordingRunner {
            fn resume_with_options(
                &self,
                thread_id: &str,
                _prompt: &str,
                last_message_path: &Path,
                _options: &CodexResumeRequestOptions,
            ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>
            {
                self.threads.borrow_mut().push(thread_id.to_string());
                if let Some(parent) = last_message_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(
                    last_message_path,
                    "干完了。\n```json\n{\"did\":\"x\",\"outputs\":[],\"status\":\"done\",\"evidence\":[]}\n```",
                );
                Ok((
                    CodexResumeRunResult {
                        exit_code: 0,
                        timed_out: false,
                        stderr_summary: None,
                    },
                    WorkflowNodeDispatchExecutionOptions {
                        readback_stats: Some(CodexDispatchReadbackStats {
                            transcript_event_count: 3,
                            transcript_target_hits: 1,
                        }),
                    },
                ))
            }
        }

        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("c1-chain");
        let path = dir.join("workflow-state.v0.json");
        // 4 线索引：thread-qdebt（seed 绑）+ 3 个任务会话线（C1 每任务绑·bind 认索引里的线）。
        let threads_json: Vec<Value> = [
            "thread-qdebt",
            "thread-c1-task-1",
            "thread-c1-task-2",
            "thread-c1-task-3",
        ]
        .iter()
        .map(|t| {
            serde_json::json!({
                "thread_id": t, "project_root": test_root, "title": format!("Session {t}"),
                "rollout_exists": true, "rollout_path": format!("/tmp/{t}.jsonl")
            })
        })
        .collect();
        let index = serde_json::json!({
            "projects": [{ "project_root": test_root }],
            "threads": threads_json
        });
        let workflow_id = seed_active_run(&path, &index, test_root);
        let pstore = project_consultation_proposal_store::load_store(&path, unix_timestamp_ms())
            .expect("pstore");
        let proposal = pstore.proposals[0].clone();
        let astore =
            plan_authorization_store::load_store(&path, unix_timestamp_ms()).expect("astore");
        let auth = astore
            .authorizations
            .iter()
            .find(|a| a.workflow_id == workflow_id && a.status == PlanAuthorizationStatus::Active)
            .expect("active auth");
        let scope = director_task_scope_from_proposal(&proposal, "codex-dev");
        let mk = |id: usize, title: &str, deps: Vec<String>| ProjectDirectorPlannedTask {
            planned_task_id: format!("planned-task:{workflow_id}:{id}"),
            title: title.to_string(),
            task_goal: format!("自包含：{title}"),
            scope: scope.clone(),
            depends_on: deps,
            acceptance_criteria: vec!["ok".to_string()],
            report_format: vec!["r".to_string()],
            status: "planned".to_string(),
            guard_result: None,
            work_item_id: None,
            workflow_node_id: None,
            task_package_id: None,
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        };
        let planned = vec![
            mk(1, "任务甲", vec![]),
            mk(2, "任务乙", vec!["任务甲".to_string()]),
            mk(3, "任务丙", vec!["任务乙".to_string()]),
        ];
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &PrepareAuthorizedAutoDispatchInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: workflow_id.clone(),
                proposal_id: proposal.proposal_id.clone(),
                authorization_id: auth.authorization_id.clone(),
                actor_id: "user-fixture".to_string(),
                planned_tasks: planned,
                expected_workflow_revision: None,
                expected_authorization_revision: None,
                chain_binds_per_task: false,
            },
        )
        .expect("prepare");
        assert_eq!(
            prepared.plan.prepared_dispatch_count, 3,
            "3 任务应全 prepared"
        );

        let creator = DistinctCreator {
            calls: Cell::new(0),
        };
        let runner = RecordingRunner {
            threads: RefCell::new(vec![]),
        };
        let readback = dir.join("readback.db");
        let outcome = run_director_task_chain_with_session_creator(
            &path,
            &index,
            &readback,
            &runner,
            test_root,
            &workflow_id,
            &prepared.plan.planned_tasks,
            50,
            &creator,
        )
        .expect("C1 链应跑完");

        // 断言①：会话工厂每任务一次 = 3。
        assert_eq!(creator.calls.get(), 3, "① creator 被调 3 次");
        assert_eq!(outcome.completed, 3, "3 任务全完成");
        // 断言②：各任务 dispatch 用各自新 thread（互异·且是新建会话号）。
        let threads = runner.threads.borrow();
        assert_eq!(threads.len(), 3, "3 任务各 dispatch 一次");
        let distinct: std::collections::BTreeSet<&String> = threads.iter().collect();
        assert_eq!(
            distinct.len(),
            3,
            "② 各任务 dispatch thread 互异：{threads:?}"
        );
        assert!(
            threads.iter().all(|t| t.starts_with("thread-c1-task-")),
            "② dispatch 用的是新建会话线：{threads:?}"
        );
        // 断言③：3 个 target_session_id 物化进各任务 artifact 且互异。
        let value = read_workflow_state_value(&path).unwrap();
        let artifacts = value["artifacts"].as_array().cloned().unwrap_or_default();
        let session_ids: std::collections::BTreeSet<String> = prepared
            .plan
            .planned_tasks
            .iter()
            .filter_map(|task| task.task_package_id.as_ref())
            .filter_map(|aid| {
                artifacts.iter().find(|a| {
                    optional_string_from(a, "artifact_id").as_deref() == Some(aid.as_str())
                })
            })
            .filter_map(|a| optional_string_from(a, "target_session_id"))
            .collect();
        assert_eq!(
            session_ids.len(),
            3,
            "③ 3 个 target_session_id 物化且互异：{session_ids:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §8·②真跑耗时（单独步·#[ignore]·固定测试项目·额度在）：真建一条会话，量「每任务 +约 1 分钟」的
    // 单次实数（N 任务链的会话开销 = N × 本数·线性）。显式 `cargo test --lib c1_session_creation_timing_real
    // -- --ignored --nocapture`。核实物：真 thread_id + 耗时打印。
    #[test]
    #[ignore = "C1 timing: really create one 先生后绑 session in the test project and print elapsed (user present, quota available)"]
    fn c1_session_creation_timing_real() {
        let started = std::time::Instant::now();
        let thread_id = ManualRelayJiaobanNewSessionCreator
            .create_initialized_session(
                "交办任务专用会话（耗时实测）：本会话只用于测量新建会话耗时。回复「已就位」即可，别改任何文件。",
                "director_chain_timing",
            )
            .expect("真建会话应成功（额度在·测试项目）");
        let elapsed_ms = started.elapsed().as_millis();
        println!("[C1_TIMING] 单次先生后绑建会话耗时 = {elapsed_ms} ms（thread {thread_id}）");
        println!(
            "[C1_TIMING] N 任务链会话总开销 ≈ N × {elapsed_ms} ms（每任务一条·线性·知情代价）"
        );
        assert!(!thread_id.trim().is_empty(), "应拿到真 thread_id");
    }

    // §4·新断言：独立[接着跑]/自动免管路（run_auto_advance_authorized_role_loop_with_session_creator）
    // 3 任务链 → 每任务先生后绑建新会话（creator 3 次）+ 各任务 dispatch 用各自新 thread 互异 + target_session_id
    // 互异物化。证「自动路已接 C1 每任务新会话」（对比 6565 拐杖测·公有壳仍跑预绑）。
    #[test]
    fn c1_auto_advance_new_conversation_path_creates_per_task_sessions() {
        struct ThreeTaskDirector;
        impl DirectorAgent for ThreeTaskDirector {
            fn plan(
                &self,
                _ctx: &ProjectContext,
                proposal: &ProjectConsultationProposal,
            ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
                let scope = director_task_scope_from_proposal(proposal, "codex-dev");
                let mk = |id: usize, title: &str, deps: Vec<String>| ProjectDirectorPlannedTask {
                    planned_task_id: format!("planned-task:{}:{id}", proposal.workflow_id),
                    title: title.to_string(),
                    task_goal: format!("自包含：{title}"),
                    scope: scope.clone(),
                    depends_on: deps,
                    acceptance_criteria: vec!["ok".to_string()],
                    report_format: vec!["r".to_string()],
                    status: "planned".to_string(),
                    guard_result: None,
                    work_item_id: None,
                    workflow_node_id: None,
                    task_package_id: None,
                    memory_packet_snapshot_id: None,
                    prepared_dispatch_id: None,
                    blocked_reasons: vec![],
                };
                Ok(vec![
                    mk(1, "任务甲", vec![]),
                    mk(2, "任务乙", vec!["任务甲".to_string()]),
                    mk(3, "任务丙", vec!["任务乙".to_string()]),
                ])
            }
        }
        struct DistinctCreator {
            calls: Cell<usize>,
        }
        impl JiaobanNewSessionCreator for DistinctCreator {
            fn create_initialized_session(&self, _t: &str, _b: &str) -> Result<String, String> {
                let n = self.calls.get() + 1;
                self.calls.set(n);
                Ok(format!("thread-c1-task-{n}"))
            }
        }
        struct RecordingRunner {
            threads: RefCell<Vec<String>>,
        }
        impl CodexResumeRunner for RecordingRunner {
            fn resume_with_options(
                &self,
                thread_id: &str,
                _p: &str,
                last_message_path: &Path,
                _o: &CodexResumeRequestOptions,
            ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>
            {
                self.threads.borrow_mut().push(thread_id.to_string());
                if let Some(parent) = last_message_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(
                    last_message_path,
                    "干完了。\n```json\n{\"did\":\"x\",\"outputs\":[],\"status\":\"done\",\"evidence\":[]}\n```",
                );
                Ok((
                    CodexResumeRunResult {
                        exit_code: 0,
                        timed_out: false,
                        stderr_summary: None,
                    },
                    WorkflowNodeDispatchExecutionOptions {
                        readback_stats: Some(CodexDispatchReadbackStats {
                            transcript_event_count: 3,
                            transcript_target_hits: 1,
                        }),
                    },
                ))
            }
        }

        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = tmp_dir("c1-auto");
        let path = dir.join("workflow-state.v0.json");
        let threads_json: Vec<Value> = [
            "thread-qdebt",
            "thread-c1-task-1",
            "thread-c1-task-2",
            "thread-c1-task-3",
        ]
        .iter()
        .map(|t| {
            serde_json::json!({
                "thread_id": t, "project_root": test_root, "title": format!("Session {t}"),
                "rollout_exists": true, "rollout_path": format!("/tmp/{t}.jsonl")
            })
        })
        .collect();
        let index = serde_json::json!({
            "projects": [{ "project_root": test_root }],
            "threads": threads_json
        });
        let workflow_id = seed_active_run(&path, &index, test_root);
        let readback = dir.join("readback.db");
        let creator = DistinctCreator {
            calls: Cell::new(0),
        };
        let runner = RecordingRunner {
            threads: RefCell::new(vec![]),
        };
        let outcome = run_auto_advance_authorized_role_loop_with_session_creator(
            &path,
            &index,
            &readback,
            &runner,
            &ThreeTaskDirector,
            test_root,
            &workflow_id,
            "tester",
            50,
            None,
            &creator,
        )
        .expect("C1 自动路应跑通");
        assert_eq!(outcome.stage, "ran", "应跑到链：{outcome:?}");
        // ① 每任务先生后绑一次。
        assert_eq!(creator.calls.get(), 3, "① 自动路每任务建新会话·3 次");
        // ② 各任务 dispatch 用各自新 thread 互异。
        let threads = runner.threads.borrow();
        assert_eq!(threads.len(), 3, "3 任务各 dispatch 一次");
        let distinct: std::collections::BTreeSet<&String> = threads.iter().collect();
        assert_eq!(distinct.len(), 3, "② dispatch thread 互异：{threads:?}");
        assert!(
            threads.iter().all(|t| t.starts_with("thread-c1-task-")),
            "② dispatch 用新建会话线：{threads:?}"
        );
        // ③ target_session_id 互异物化（全 artifact 里非空 target_session_id 恰 3 个互异）。
        let value = read_workflow_state_value(&path).unwrap();
        let session_ids: std::collections::BTreeSet<String> = value["artifacts"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|a| optional_string_from(a, "target_session_id"))
            .collect();
        assert_eq!(
            session_ids.len(),
            3,
            "③ 3 个 target_session_id 互异物化：{session_ids:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
