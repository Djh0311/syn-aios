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
}

// ===== v0 静态主管档案 =====
const DIRECTOR_V0_PROFILE: &str = r#"你是「项目主管」。
职责:把已授权的方案拆成可派发给 worker 的具体任务。每个任务定清:做什么(objective)、依赖顺序(depends_on)、验收标准(acceptance_criteria)、汇报格式(report_format)。
铁律·自包含(最重要):**worker 在干净隔离上下文里执行,只看到这个任务的 objective 字符串——看不到这份方案、看不到别的任务、不能按需读文件。** 所以每个 objective 必须把执行所需的一切**写全进去**:目标文件的**完整路径**、**要写的具体内容**、依据的事实/数据**原样抄进来**。**绝不许写"按已注入方案/参见上文/见上一步/如方案所述"——worker 根本看不到那些。** 你已拿到方案,你的职责就是把它**翻译成 worker 只看 objective 就能独立干完的自包含指令**。
铁律·落地:只依据已注入的方案正文和项目上下文拆,不假设未注入的内容存在。任务对得上方案 objective,不加方案没授权的事。
边界:只读、只规划、不执行、不自己派发。真派发由用户审过后走授权闸——你只产计划。
风格:任务粒度适中、依赖清晰、可验收;不堆废话。"#;

fn director_build_prompt(ctx: &ProjectContext, proposal: &ProjectConsultationProposal) -> String {
    let mut p = String::new();
    p.push_str(DIRECTOR_V0_PROFILE);
    p.push_str("\n\n===== 已授权方案（要拆的就是它）=====\n");
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
    p.push_str(
        r#"
===== 怎么拆 =====
把已授权方案拆成有序的 worker 任务(通常 1-6 个)。只依据上面注入的方案+文档,不假设未注入内容。
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
        let raw =
            codex_local_runner::readonly_codex_consult(&ctx.project_root, &prompt, self.timeout_ms)?;
        parse_director_plan(&raw, proposal)
    }
}

// ===== S3 主管→worker 链驱动（薄·按 depends_on 拓扑序跑 prepared 的 planned_tasks）=====
// 复用 execute_project_workflow_node_at（**S1 闸/沙箱·每节点 path-lock**）+ workflow_chain_topological_order
// （现成拓扑·都在 crate root，chain controller 本体 0-diff）+ prepare 产的 work_items。
// 护栏：拓扑序 / 失败即停 / runaway 上限 / 审计（每节点经 execute 写 dispatch 记录）。
// 同-role 多任务共享 1 节点没关系——每次 execute 用**该任务自己的 work_item**（objective 各异）按序真跑。
// 注：可中断（mid-chain async stop）是 chain controller 长链才需的护栏，本薄驱动是短链同步执行、暂不含。
pub(crate) struct DirectorChainOutcome {
    pub(crate) total: usize,
    pub(crate) dispatched: usize,
    pub(crate) completed: usize,
    pub(crate) stopped_reason: Option<String>,
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
    let titles: Vec<String> = tasks.iter().map(|task| task.title.clone()).collect();
    let edges: Vec<(String, String)> = tasks
        .iter()
        .flat_map(|task| {
            task.depends_on
                .iter()
                .map(|dep| (dep.clone(), task.title.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    let order = workflow_chain_topological_order(&titles, &edges)?;
    let total = tasks.len();
    let mut dispatched = 0usize;
    let mut completed = 0usize;
    for title in &order {
        let task = match tasks.iter().find(|task| &task.title == title) {
            Some(task) => task,
            None => continue,
        };
        // 只跑 prepare 授权过的任务（有 work_item + node）；被 guard 拦 / needs_binding 的跳过。
        let (node_id, work_item_id) =
            match (task.workflow_node_id.clone(), task.work_item_id.clone()) {
                (Some(node), Some(work_item)) => (node, work_item),
                _ => continue,
            };
        if dispatched >= max_tasks {
            return Ok(DirectorChainOutcome {
                total,
                dispatched,
                completed,
                stopped_reason: Some(format!("runaway_cap_reached:{max_tasks}")),
            });
        }
        dispatched += 1;
        let request = ProjectWorkflowNodeRunRequest {
            project_root: project_root.to_string(),
            node_id,
            work_item_id,
            workflow_id: Some(workflow_id.to_string()),
        };
        match execute_project_workflow_node_at(path, index, readback_db_path, runner, &request) {
            Ok(result) if result.dispatch.state == "completed" => {
                completed += 1;
            }
            Ok(result) => {
                return Ok(DirectorChainOutcome {
                    total,
                    dispatched,
                    completed,
                    stopped_reason: Some(format!(
                        "fail_stop:node_not_completed:{title}:{}",
                        result.dispatch.state
                    )),
                });
            }
            Err(error) => {
                return Ok(DirectorChainOutcome {
                    total,
                    dispatched,
                    completed,
                    stopped_reason: Some(format!("fail_stop:node_error:{title}:{error}")),
                });
            }
        }
    }
    Ok(DirectorChainOutcome {
        total,
        dispatched,
        completed,
        stopped_reason: None,
    })
}
