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

// ===== v0 静态主管档案 =====
const DIRECTOR_V0_PROFILE: &str = r#"你是「项目主管」。
职责:把已授权的方案拆成可派发给 worker 的具体任务。每个任务定清:做什么(objective)、依赖顺序(depends_on)、验收标准(acceptance_criteria)、汇报格式(report_format)。
铁律·自包含(最重要):**worker 在干净隔离上下文里执行,只看到这个任务的 objective 字符串——看不到这份方案、看不到别的任务、不能按需读文件。** 所以每个 objective 必须把执行所需的一切**写全进去**:目标文件的**完整路径**、**要写的具体内容**、依据的事实/数据**原样抄进来**。**绝不许写"按已注入方案/参见上文/见上一步/如方案所述"——worker 根本看不到那些。** 你已拿到方案,你的职责就是把它**翻译成 worker 只看 objective 就能独立干完的自包含指令**。
铁律·落地:只依据已注入的方案正文和项目上下文拆,不假设未注入的内容存在。任务对得上方案 objective,不加方案没授权的事。
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
**每个 objective 必须自包含**:把目标文件的完整路径、要写的具体内容、依据的事实**原样写进 objective**——worker 只看这段、不看方案/不看别的任务也能独立干完。**绝不写"参见方案/见上文/如上所述/见上一步"。**
report_format 写清 worker 该**结构化返回**什么(做了啥 / 产出在哪 / 成败),好让链/主管 parse 了往下走。
在最后输出且仅输出一个 ```json 代码块,是一个任务数组,严格这个结构:
[
  {
    "title": "任务名",
    "objective": "自包含完整指令:做什么 + 目标文件完整路径 + 要写的具体内容 + 依据(worker 只看这段就能干,不引用方案/别任务)",
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
    #[serde(default)]
    objective: String,
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
                objective: task.objective,
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

// ===== S3 主管→worker 链驱动（薄·按 depends_on 拓扑序跑 prepared 的 planned_tasks）=====
// 复用 execute_project_workflow_node_at（**S1 闸/沙箱·每节点 path-lock**）+ workflow_chain_topological_order
// （现成拓扑）+ chain controller 的链记录/停链/审计 helper（ensure_chain_run_record / set_chain_node_state /
// chain_run_stop_requested / finalize_chain_run / append_chain_audit）——全在 crate root 同模块（include!），
// 只调不改 → `workflow_chain_controller.rs` **本体 byte-0-diff**，无需开可见性。
// 4 护栏全在：① runaway 上限（max_nodes=min(max_tasks,任务数,硬顶50)）② **可中断**（每任务边界 read-fresh
// 查 stop_requested → 停；现成 `stop_project_workflow_chain` 命令按 workflow_id+running 能找到本驱动的链记录）
// ③ 审计（链起/每任务 start·done·skip·fail/链停·完成·失败 都进 audit_events）④ 可回滚（起链前 backup +
// execute 每派发 backup）。同-role 多任务共享 1 节点没关系——每次 execute 用**该任务自己的 work_item**
// （objective 各异）按序真跑；链记录的「节点」按 **planned_task_id** 编址（≠工作流 node_id，避免同-role 撞键）。
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
    pub(crate) warnings: Vec<String>,
    pub(crate) stopped_reason: Option<String>,
}

// 2.4：重试前把 work_item 走**现成合法跳转**复位到 ready_to_dispatch（首次失败推离了它）——用现成
// update_work_item_state_at（限默认工作流），非默认工作流复位不成则返回 false（不重试·不硬闯状态机）。
// 逐步 fire（running→failed→needs_changes→ready_to_dispatch·非法跳转各步自忽略），以末步是否到 ready_to_dispatch 为准。
fn reset_work_item_for_retry(
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
    let _ = step("failed"); // 若卡在 running：running→failed（已 failed 则非法·忽略）
    let _ = step("needs_changes"); // failed/timed_out → needs_changes
    step("ready_to_dispatch") // needs_changes → ready_to_dispatch（末步·返回是否复位成功）
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
                set_chain_node_state(
                    &mut after,
                    &chain_run_id,
                    task_id,
                    "completed",
                    Some(&dispatch_id),
                    None,
                );
                update_node_state_for_id(&mut after, &task_level_node_id, "completed", &ts_done)?;
                append_chain_audit(
                    &mut after,
                    &chain_run_id,
                    workflow_id,
                    "workflow_chain_node_completed",
                    "running",
                    "completed",
                    &ts_done,
                    &format!(
                        "薄链驱动：任务「{}」真派发成功（dispatch {dispatch_id}）",
                        task.title
                    ),
                )?;
                write_validated_workflow_state(path, &after)?;
                completed += 1;

                // fix·worker 回程契约：任务完成后读 worker 最后消息全文 → 解析 → best-effort 落库 → 带摘要。
                // **只归档不驱动**：无论解析/落库成败，任务恒算 completed（state 下面写死、不 retry/不停链/不改迁移）。
                let last_message_full = result
                    .dispatch
                    .last_message_path
                    .as_deref()
                    .and_then(|last_message_path| std::fs::read_to_string(last_message_path).ok())
                    .unwrap_or_default();
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
                steps.push(DirectorChainStep {
                    planned_task_id: task_id.clone(),
                    title: task.title.clone(),
                    state: "completed".to_string(),
                    report_summary: report_outcome.report_summary,
                    report_warning: report_outcome.report_warning,
                    report_status: report_outcome.report_status,
                });
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
    Ok(DirectorChainOutcome {
        total,
        dispatched,
        completed,
        skipped,
        chain_run_id,
        steps,
        warnings,
        stopped_reason: None,
    })
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
        run_director_task_chain(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &request.project_root,
            &request.workflow_id,
            &request.planned_tasks,
            request.max_nodes.unwrap_or(50),
        )
    })
    .await
    .map_err(|error| format!("主管链执行线程异常：{error}"))?
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
    // fix9·[接着跑]口同款守卫：空写根 active 授权（16:48/16:55 已有两份残留在盘）→ 人话停，
    // 不进 LM 拆/prepare（存量空授权从「坑」变「哑」，零 store 手术）。started 审计之前拦=没开始就不记 started。
    if active.scope.allowed_write_roots.is_empty() {
        let message = "这单的授权没带可执行范围（多半是方案被判成了纯建议），接着跑也只会空转。请点[重新出方案]把要动手的内容说清楚——写范围由系统自动装配，不需要你手填。".to_string();
        let _ = append_role_loop_auto_advance_audit(
            path,
            workflow_id,
            actor_id,
            "role_loop_auto_advance_stopped",
            &format!("接着跑口拒空写根授权（未进拆任务/prepare）：{message}"),
        );
        return Err(message);
    }
    append_role_loop_auto_advance_audit(
        path,
        workflow_id,
        actor_id,
        "role_loop_auto_advance_started",
        "已查到 active 方案授权，开始授权范围内自动推进：拆任务 → prepare →（没绑/越界则停）→ 链跑。",
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
        };
        let prepared = prepare_authorized_auto_dispatch_for_index_at(path, index, &prepare_input)?;
        let planned_task_count = prepared.plan.planned_task_count;
        let prepared_count = prepared.plan.prepared_dispatch_count;
        let needs_binding_count = prepared.plan.needs_binding_count;
        let blocked_count = prepared.plan.blocked_count;
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
                // fix9 改口：老话教用户「在方案里补上」写范围——档位时代写范围由系统装配、用户没处补（死胡同）。
                "blocked",
                format!(
                    "有任务超出方案授权范围被阻断{reasons_text}——这单的授权没带可执行范围（多半是方案被判成了纯建议）。请点[重新出方案]把要动手的内容说清楚——写范围由系统自动装配，不需要你手填。"
                ),
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
        let outcome = run_director_task_chain(
            path,
            index,
            readback_db_path,
            runner,
            project_root,
            workflow_id,
            &prepared.plan.planned_tasks,
            max_nodes,
        )?;
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
        run_auto_advance_authorized_role_loop(
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
    pub(crate) session_choice: String, // "existing" | "new"（new=方案a 先生后绑）
    #[serde(default)]
    pub(crate) session_id: Option<String>, // session_choice=existing 时的现有 Codex 会话 thread_id
    #[serde(default)]
    pub(crate) actor_id: Option<String>,
    #[serde(default)]
    pub(crate) max_nodes: Option<usize>,
    // 2.2 所批即所跑：前端回传「用户批过的那份图」（预拆→审→回传）。Some=原样跑不重拆；None/缺=批后 LM 拆（现状）。
    #[serde(default)]
    pub(crate) approved_planned_tasks: Option<Vec<ProjectDirectorPlannedTask>>,
}

// 内层（同步·spawn_blocking 里调；可单测·stub 咨询/主管/链/新会话出生口）。
fn run_confirm_and_start_authorized_run_inner(
    path: &std::path::Path,
    index: &Value,
    readback_db_path: &std::path::Path,
    runner: &dyn CodexResumeRunner,
    director: &dyn DirectorAgent,
    session_creator: &dyn JiaobanNewSessionCreator,
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
    // fix9·开工口守卫（确定性·零 LM 依赖·2026-07-07 16:48/16:55 两撞）：tier-1 咨询偶发不交
    // execution_scope → 分流忠实映射成纯建议只读方案（写根空）。这种方案点[允许并开始]只会建
    // 空写根授权 → prepare 逐任务拦 → 空转。在人闸校验之后、**建授权之前**人话拒——不建授权、
    // 不绑会话、不起链（16:48 那种空授权垃圾不再入库）。留档走现有 stopped 事件族（不新开）。
    // 注：当前档位世界「写根空 ⇔ 纯建议」；将来若出现「只读但要跑检查」的新档位形态，本守卫须随分流一起升级。
    if proposal.scope_draft.allowed_write_roots.is_empty() {
        let message = "这份方案是纯建议（咨询判定不需要改文件），没有可执行范围，开工只会空转。想让 AI 动手：点[重新出方案]，把要改什么说清楚（带上文件名/功能名更稳）。".to_string();
        let _ = append_role_loop_auto_advance_audit(
            path,
            &workflow_id,
            &actor_id,
            "role_loop_auto_advance_stopped",
            &format!("开工口拒纯建议方案（写根空·未建授权未起链）：{message}"),
        );
        return Err(message);
    }
    // 2. 记录用户确认（现成 record_decision·Confirm·actor=用户）→ 建授权。
    let confirmed = project_consultation_proposal_store::record_decision(
        path,
        &RecordProjectConsultationProposalDecisionInput {
            project_root: request.project_root.clone(),
            proposal_id: request.proposal_id.clone(),
            actor_id: actor_id.clone(),
            decision: ProjectConsultationProposalDecisionKind::Confirm,
            summary: "用户点[允许并开始]：确认方案。".to_string(),
            expected_proposal_store_revision: Some(proposal_store_revision),
            expected_plan_authorization_store_revision: None,
        },
        timestamp_ms,
        &format!("confirm-and-start-confirm:{}", unix_timestamp_nanos()),
        &format!("confirm-and-start-auth:{}", unix_timestamp_nanos()),
        &format!("confirm-and-start-auth-user:{}", unix_timestamp_nanos()),
    )?;
    // fix3 2.2：record_decision（确认）**成功之后**的任何失败（授权提取/边界复核/建会话/绑会话）→ 先 append
    // stopped 审计（人话）再返回 Err（治「今晚审计只有 started、之后空白」）。step5 auto_advance 由它自己留档、
    // 不含在此（避免双记）；record_decision 本身失败=确认闸没过、按包不记（在此之前）。只 append、不改状态、不吞错。
    // 方案a：new 分支建成会话后产一句人话说明（让等待有名目），随 outcome.warnings 带出。
    let mut new_session_notice: Option<String> = None;
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
                summary: "用户点[允许并开始]：同时作全局边界批准（Phase A·用户演全局主管）。"
                    .to_string(),
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
        // 4. 绑会话（2.3）：existing → 绑传入的现有 Codex 会话（绑失败即停·停因人话·不静默）；
        //    new → 方案a「先生后绑」：先经现成 relay 单次路径真建一条会话（固定测试项目·初始化消息）→
        //    回执取 thread_id → 走 existing **同一套绑定**；失败即停（外层留档），不静默回落 existing。
        match request.session_choice.as_str() {
            "existing" => {
                let session_id = request.session_id.as_deref().ok_or_else(|| {
                    "session_choice=existing 需给 session_id（要绑的现有 Codex 会话）。".to_string()
                })?;
                let node_id = format!("{workflow_id}:node:codex-dev");
                bind_workflow_node_codex_session_for_index_at(
                    path,
                    index,
                    &WorkflowNodeSessionBindRequest {
                        project_root: request.project_root.clone(),
                        node_id,
                        work_item_id: None,
                        thread_id: session_id.to_string(),
                    },
                )
                .map_err(|error| format!("绑定现有会话失败（会话没找到/不可用）：{error}"))?;
            }
            "new" => {
                let initialization_text = format!(
                    "交办新会话初始化：这条会话专用于承接方案「{}」的 worker 任务（工作流 {workflow_id}）。现在先不要改动任何文件，回复「已就位」即可；具体任务稍后会逐条发来。",
                    confirmed.proposal.title
                );
                let thread_id = session_creator
                    .create_initialized_session(&initialization_text, &actor_id)
                    .map_err(|error| format!("新会话没建起来：{error}"))?;
                let node_id = format!("{workflow_id}:node:codex-dev");
                let bind_request = WorkflowNodeSessionBindRequest {
                    project_root: request.project_root.clone(),
                    node_id,
                    work_item_id: None,
                    thread_id: thread_id.clone(),
                };
                // 同一套绑定机器（existing 同款）；只多一层「等 codex 落库可见」的重试——
                // 仅对「会话不在当前索引内」这一类可见性时差重试，其余错误原样即停。
                let bind_started = std::time::Instant::now();
                loop {
                    match bind_workflow_node_codex_session_for_index_at(path, index, &bind_request)
                    {
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
                new_session_notice = Some(format!(
                    "已为这单活新建会话（初始化 ~1 分钟·thread {thread_id}）。"
                ));
            }
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
    // 5. 自动推进（现成 auto_advance 内层·resolve active 授权 →〔2.2 有已批图则跳过 LM 原样跑〕→ prepare →
    //    起链前复查授权 → 链）。approved_planned_tasks=Some 时所批即所跑（预拆给用户看的那份=真跑的那份）。
    let mut outcome = run_auto_advance_authorized_role_loop(
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
    )?;
    if let Some(notice) = new_session_notice {
        outcome.warnings.push(notice);
    }
    Ok(outcome)
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
    struct PanicDirector;
    impl DirectorAgent for PanicDirector {
        fn plan(
            &self,
            _ctx: &ProjectContext,
            _proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            panic!("fix9 守卫应在主管 LM 拆任务之前拦住");
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

    fn stopped_audit_reasons(state: &serde_json::Value) -> Vec<String> {
        state["audit_events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .filter(|event| event["event_type"] == "role_loop_auto_advance_stopped")
                    .filter_map(|event| event["reason"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    // §4①：纯建议方案（写根空）→ 合流拒·人话对·授权店零新增·方案仍 Pending·stopped 留档·
    // 三个 panic 桩全没炸（没建授权没绑会话没起链）。
    #[test]
    fn fix9_confirm_rejects_advice_only_proposal_before_authorization() {
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
        let err = run_confirm_and_start_authorized_run_inner(
            &path,
            &serde_json::json!({"projects": []}),
            &dir.join("readback.sqlite"),
            &PanicRunner,
            &PanicDirector,
            &PanicCreator,
            &request,
        )
        .expect_err("纯建议方案应被开工口拒");
        assert!(
            err.contains("纯建议") && err.contains("重新出方案"),
            "人话应点名纯建议并指对路：{err}"
        );
        // 授权店零新增（16:48 那种空授权垃圾不再入库）。
        let auth_store =
            plan_authorization_store::load_store(&path, unix_timestamp_ms()).expect("auth store");
        assert!(
            auth_store.authorizations.is_empty(),
            "不许建授权：{:?}",
            auth_store.authorizations.len()
        );
        // 方案仍 Pending（record_decision 没跑·人闸语义没动）。
        let proposal_store =
            project_consultation_proposal_store::load_store(&path, unix_timestamp_ms())
                .expect("proposal store");
        assert!(
            matches!(
                proposal_store.proposals[0].status,
                ProjectConsultationProposalStatus::PendingUserConfirmation
            ),
            "方案应仍是待确认（守卫在确认之前）"
        );
        // 留档走现有 stopped 事件族。
        let state = read_state_json(&path);
        let reasons = stopped_audit_reasons(&state);
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("开工口拒纯建议")),
            "应留 stopped 审计：{reasons:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // §4②：空写根 active 授权（盘上残留形态·全程生产 API 造）+ [接着跑] → 人话停·不进拆任务/prepare
    // （panic 桩没炸）·不记 started·记 stopped。
    #[test]
    fn fix9_auto_advance_rejects_empty_write_root_active_authorization() {
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
        // [接着跑] → 守卫人话停。
        let err = run_auto_advance_authorized_role_loop(
            &path,
            &serde_json::json!({"projects": []}),
            &dir.join("readback.sqlite"),
            &PanicRunner,
            &PanicDirector,
            test_root,
            &proposal.workflow_id,
            "user-fixture",
            10,
            None,
        )
        .expect_err("空写根授权应被接着跑口拒");
        assert!(
            err.contains("没带可执行范围") && err.contains("重新出方案"),
            "人话应指对路：{err}"
        );
        let state = read_state_json(&path);
        let reasons = stopped_audit_reasons(&state);
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("接着跑口拒空写根授权")),
            "应留 stopped 审计：{reasons:?}"
        );
        let started = state["audit_events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .any(|event| event["event_type"] == "role_loop_auto_advance_started")
            })
            .unwrap_or(false);
        assert!(!started, "守卫在 started 之前拦=没开始就不记 started");
        let _ = fs::remove_dir_all(dir);
    }

    // §4③：正常档位方案（execution_scope Some → 档位写根非空）不触守卫——走到人闸之后的正常路径
    // （本测只验「守卫不误伤」：同请求打到绑会话步才因假会话失败，而非被纯建议拒）。
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
        let err = run_confirm_and_start_authorized_run_inner(
            &path,
            &serde_json::json!({"projects": [{"project_root": test_root}]}),
            &dir.join("readback.sqlite"),
            &PanicRunner,
            &PanicDirector,
            &PanicCreator,
            &request,
        )
        .expect_err("会走到绑会话步并因假会话失败（证明没被纯建议守卫拦）");
        assert!(
            !err.contains("纯建议"),
            "正常档位方案不得被纯建议守卫误伤：{err}"
        );
        assert!(
            err.contains("绑定现有会话失败"),
            "应死在绑会话步（守卫之后的正常路径）：{err}"
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
                objective: format!("自包含目标：{title}"),
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

    // 脚本化 runner：按序弹出行为（"ok"=完成+契约口供 / "timeout"=超时被杀 / "fail"=普通失败 /
    // "provider_err"=供给类 Err / 脚本耗尽默认 ok）。
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
            objective: "自包含".to_string(),
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
}
