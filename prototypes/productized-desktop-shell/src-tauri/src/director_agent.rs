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
    let chain_run_id =
        ensure_chain_run_record(&mut value, &pid, workflow_id, &order_task_ids, max_nodes, &start_ts)?;
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
                Some(&format!("status={}（非 prepared，未授权派发）", task.status)),
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
            });
            continue;
        }
        // prepared 理应有 node+work_item（annotate 无条件设）；防御：缺则记 skipped、不派。
        let (node_id, work_item_id) = match (task.workflow_node_id.clone(), task.work_item_id.clone())
        {
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
                    &format!("任务「{}」status=prepared 但缺 node/work_item，跳过。", task.title),
                )?;
                write_validated_workflow_state(path, &current)?;
                skipped += 1;
                steps.push(DirectorChainStep {
                    planned_task_id: task_id.clone(),
                    title: task.title.clone(),
                    state: "skipped".to_string(),
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
        let outcome =
            execute_project_workflow_node_at(path, index, readback_db_path, runner, &request);

        // 重读（execute 写过文件，避免覆盖它的写入）。
        let mut after = read_workflow_state_value(path)?;
        let ts_done = unix_timestamp_string();
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
                append_chain_audit(
                    &mut after,
                    &chain_run_id,
                    workflow_id,
                    "workflow_chain_node_completed",
                    "running",
                    "completed",
                    &ts_done,
                    &format!("薄链驱动：任务「{}」真派发成功（dispatch {dispatch_id}）", task.title),
                )?;
                write_validated_workflow_state(path, &after)?;
                completed += 1;
                steps.push(DirectorChainStep {
                    planned_task_id: task_id.clone(),
                    title: task.title.clone(),
                    state: "completed".to_string(),
                });
            }
            // 失败即停（护栏·不自动重试/不跳过，防在老失败任务上打转）。
            other => {
                let fail_msg = match &other {
                    Ok(result) => format!("worker 派发未完成（state={}）", result.dispatch.state),
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
                    &format!("任务「{}」失败，已停链（失败即停、不自动重试）：{fail_msg}", task.title),
                )?;
                write_validated_workflow_state(path, &after)?;
                steps.push(DirectorChainStep {
                    planned_task_id: task_id.clone(),
                    title: task.title.clone(),
                    state: "failed".to_string(),
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
            "无 active 方案授权：请先确认方案 + 全局边界复核（自动推进不创建、不跳过授权）。".to_string()
        })?;
    let proposal_id = active
        .source_proposal_id
        .clone()
        .ok_or_else(|| "active 授权缺 source_proposal_id；无法自动推进。".to_string())?;
    let authorization_id = active.authorization_id.clone();
    let auth_revision = store.revision;
    append_role_loop_auto_advance_audit(
        path,
        workflow_id,
        actor_id,
        "role_loop_auto_advance_started",
        "已查到 active 方案授权，开始授权范围内自动推进：拆任务 → prepare →（没绑/越界则停）→ 链跑。",
    )?;
    // 2. 主管 LM 拆任务（授权范围内）：加载 ctx + active 授权对应的已确认方案 → director.plan
    //    （真主管 LM·CliDirectorAgent；stub 测试注入假 director 不起 codex）。
    let ctx = load_project_context(project_root)?;
    let proposal_store = project_consultation_proposal_store::load_store(path, timestamp_ms)?;
    let proposal = proposal_store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == proposal_id)
        .ok_or_else(|| format!("找不到 active 授权对应的已确认方案：{proposal_id}"))?;
    let planned_tasks = director.plan(&ctx, proposal)?;
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
        let (stage, message) = if blocked_count > 0 {
            (
                "blocked",
                "有越界任务被阻断（超授权范围）；停下等用户处理，不自动推进。".to_string(),
            )
        } else if needs_binding_count > 0 {
            (
                "needs_binding",
                "需先给 codex-dev 节点绑一条 Codex 会话再自动推进（本命令不自动绑会话）。".to_string(),
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
        });
    }
    // 5. prepared 出来 → 跑 worker 链（四护栏·入口 path-lock 圈测试项目·失败即停）。
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
    })
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
        )
    })
    .await
    .map_err(|error| format!("自动推进执行线程异常：{error}"))?
}
