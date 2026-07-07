// S3 agent 智能层·咨询第一刀（v0·tier-1·CLI）。
// 咨询 = 借 codex 的脑 + 本地 harness（v0 静态档案 + ProjectContext 注入 + 只读 confinement）。
// 死线：咨询 codex 结构性只读（codex_local_runner::readonly_codex_consult，read-only 沙箱·写盘根空·
// command_plan_for 不改），**不走 worker 执行闸**（只读 ≠ 执行，但结构性只读挡死写/跑命令）；产出喂 C1。
// 本文件 include! 进 crate root（同 commands.rs/types.rs）：直接用 root 的 C1 类型；std 用全限定避免 use 撞。

// ===== 契约缝（稳定 trait，下游循环只认它）=====

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationRisk {
    pub(crate) severity: String,
    pub(crate) summary: String,
    pub(crate) mitigation: String,
}

// 执行范围（像开发任务包那样）：咨询在方案里**自己提出**下游执行需要的写范围/目标文件/工具/检查。
// 可空：纯问答咨询不需要下游改任何东西 → None。**后端只忠实透传·绝不默认/兜底缺失的写范围（用户硬约束）。**
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct ConsultationExecutionScope {
    pub(crate) write_roots: Vec<String>, // 写范围：下游执行可写的目录（须在被咨询项目根内）
    pub(crate) target_files: Vec<String>, // 目标文件：预期改动的具体文件·相对项目根（细粒度·可空）
    pub(crate) tools: Vec<String>, // 工具：下游 worker 需要的能力（要写就得含写能力，如 write_file/apply_patch）
    pub(crate) checks: Vec<String>, // 验收检查：怎么验（如 cargo test / npm test）
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationProposal {
    pub(crate) user_goal: String,
    pub(crate) goal_summary: String,
    pub(crate) scope_note: String,
    pub(crate) reasoning: Vec<String>,
    pub(crate) risks: Vec<ConsultationRisk>,
    pub(crate) must_stop_points: Vec<String>,
    pub(crate) next_steps: Vec<String>,
    // None = 纯咨询/只读·不需要下游改任何东西；Some = 咨询判定要下游真改东西·带执行范围。
    pub(crate) execution_scope: Option<ConsultationExecutionScope>,
    // 交办·刀2 2.5：咨询判「这活是否值得拆成多步工作流」（复杂活 true·简单/纯咨询 false）。UI 用它决定图区出不出现。
    pub(crate) suggest_workflow: bool,
}

pub(crate) trait ConsultantAgent {
    fn consult(&self, ctx: &ProjectContext, question: &str)
        -> Result<ConsultationProposal, String>;
}

// ===== ProjectContext（注入·策展核心）=====

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectContext {
    pub(crate) project_root: String,
    pub(crate) project_name: String,
    pub(crate) entry_document: Option<String>,
    pub(crate) document_map: Vec<String>,
    // tier-1 策展核心：注入文档正文（codex exec 这模式不 on-demand 读、只啃注入 → 靠它喂全文）。(相对路径, 正文)
    pub(crate) injected_documents: Vec<(String, String)>,
    pub(crate) version_signal: String,
    pub(crate) blackboard_summary: Option<String>,
    pub(crate) memory_summary: Option<String>,
}

// curated core：root 或 docs/ 下找入口文档（README/CURRENT/index，因项目而异）。返回 (相对路径, 全文)。
fn consultant_find_entry_document(root: &std::path::Path) -> Option<(String, String)> {
    let candidates = [
        "README.md",
        "docs/README.md",
        "CURRENT.md",
        "docs/CURRENT.md",
        "index.md",
        "docs/index.md",
        "readme.md",
    ];
    for rel in candidates {
        if let Ok(text) = std::fs::read_to_string(root.join(rel)) {
            if !text.trim().is_empty() {
                return Some((rel.to_string(), text));
            }
        }
    }
    None
}

// 文档/结构地图：递归收 .md（限深度/数量防爆；跳 . 开头目录=不碰 .git/.codex/.claude）。
fn consultant_build_document_map(root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    consultant_collect_md(root, root, &mut out, 0);
    out.sort();
    out.dedup();
    out
}

fn consultant_collect_md(
    dir: &std::path::Path,
    root: &std::path::Path,
    out: &mut Vec<String>,
    depth: usize,
) {
    if depth > 4 || out.len() > 200 {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            consultant_collect_md(&path, root, out, depth + 1);
        } else if name.ends_with(".md") {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().to_string());
            }
        }
    }
}

// 最新信号：git 优先；无 git → 全项目最新 mtime（防御式降级，spec §3A）。
fn consultant_version_signal(root: &std::path::Path) -> String {
    if let Ok(output) = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("log")
        .arg("-1")
        .arg("--format=%h %ci")
        .output()
    {
        if output.status.success() {
            let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !line.is_empty() {
                return format!("git:{line}");
            }
        }
    }
    let mut latest: Option<std::time::SystemTime> = None;
    consultant_collect_latest_mtime(root, &mut latest, 0);
    match latest.and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok()) {
        Some(d) => format!("mtime:{}", d.as_secs()),
        None => "no_signal".to_string(),
    }
}

fn consultant_collect_latest_mtime(
    dir: &std::path::Path,
    latest: &mut Option<std::time::SystemTime>,
    depth: usize,
) {
    if depth > 4 {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            consultant_collect_latest_mtime(&path, latest, depth + 1);
        } else if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
            if latest.map_or(true, |cur| modified > cur) {
                *latest = Some(modified);
            }
        }
    }
}

fn consultant_first_heading(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("# ")
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty())
    })
}

// tier-1 策展核心：把文档地图里的 .md 正文读进来注入（codex exec 这模式不 on-demand 读，靠注入喂全文）。
// 防爆：每篇截断 20000 字、合计 150000 字；被咨询项目只读（只读内容、绝不写）。tier-2 才改 on-demand 工具。
fn consultant_load_documents(
    root: &std::path::Path,
    document_map: &[String],
) -> Vec<(String, String)> {
    const PER_DOC_CHARS: usize = 20_000;
    const TOTAL_CHARS: usize = 150_000;
    let mut out = Vec::new();
    let mut total = 0usize;
    for rel in document_map {
        if total >= TOTAL_CHARS {
            break;
        }
        let raw = match std::fs::read_to_string(root.join(rel)) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let content = if raw.chars().count() > PER_DOC_CHARS {
            let mut truncated: String = raw.chars().take(PER_DOC_CHARS).collect();
            truncated.push_str("\n…(本文档过长已截断)…");
            truncated
        } else {
            raw
        };
        total += content.chars().count();
        out.push((rel.clone(), content));
    }
    out
}

/// 刀B·记忆召回：从正式记忆 store（workflow_state_path 侧车）取本项目 + 活跃态记忆填 memory_summary。
/// **只读**·store 读失败/坏 → None（静默降级·召回是增益不是闸·绝不挡咨询）。
fn recall_project_memory_summary_at(
    state_path: &std::path::Path,
    project_root: &str,
) -> Option<String> {
    let store = formal_memory_store::load_store(state_path, &unix_timestamp_string()).ok()?;
    recall_from_store(&store, project_root)
}

/// 纯逻辑：从已 load 的 store filter「本项目 project_id 匹配 + 活跃态」，按更新时间倒序顶格 5 条，
/// 渲染人话行「[类型] claim——body 首行」。空 → None。
fn recall_from_store(store: &FormalMemoryStoreV1, project_root: &str) -> Option<String> {
    let target = project_id(project_root);
    let mut records: Vec<&MemoryRecord> = store
        .records
        .iter()
        .filter(|record| {
            record.status == MemoryLifecycleStatus::MemoryActive
                && record.scope.project_id.as_deref() == Some(target.as_str())
        })
        .collect();
    records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    records.truncate(5);
    if records.is_empty() {
        return None;
    }
    let lines: Vec<String> = records
        .iter()
        .map(|record| {
            let body_first = record.body.lines().next().unwrap_or("").trim();
            format!(
                "[{}] {}——{}",
                record.memory_type,
                record.claim.trim(),
                body_first
            )
        })
        .collect();
    Some(lines.join("\n"))
}

// 装配 ProjectContext：有啥塞啥、不假设齐全（防御式降级）。黑板→None（无工作台数据）；
// 记忆→刀B 召回：从正式记忆 store 侧车填 memory_summary（只读·失败静默 None·绝不挡咨询）。
pub(crate) fn load_project_context(project_root: &str) -> Result<ProjectContext, String> {
    let root = std::path::Path::new(project_root);
    if !root.is_dir() {
        return Err(format!("被咨询项目目录不存在或不可读：{project_root}"));
    }
    let entry_document = consultant_find_entry_document(root).map(|(_, text)| text);
    let document_map = consultant_build_document_map(root);
    let injected_documents = consultant_load_documents(root, &document_map);
    let version_signal = consultant_version_signal(root);
    let project_name = entry_document
        .as_deref()
        .and_then(consultant_first_heading)
        .unwrap_or_else(|| {
            root.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| project_root.to_string())
        });
    Ok(ProjectContext {
        project_root: project_root.to_string(),
        project_name,
        entry_document,
        document_map,
        injected_documents,
        version_signal,
        blackboard_summary: None,
        // 纯装配：memory_summary 由各调用方用**手里的真实 path** 填（不在此死锚 default_workflow_state_path——
        // 本仓「死锚默认不穿真值」两次前科：C4 默认工作流 e2e 卡点 / update_work_item_state_at:477 绕行）。
        memory_summary: None,
    })
}

// ===== v0 静态档案（spec §3C，写死·先不 derive）=====
const CONSULTANT_V0_PROFILE: &str = r#"你是「项目咨询」。
职责:解答用户对这个项目的问题、帮定大方向。你不主导执行(那是项目主管)、不自己改任何东西。
铁律·落地:每个判断都必须基于你真读到的项目状态。不确定就用工具去读——不许假设某文件/状态/历史存在。结论里引用读到的具体依据。
边界:只读、不写、不执行。你**永不触发执行、也永不自己写**。但你可以(该有时必须)在方案里【提名下游执行需要的写范围/工具】——那是交用户授权、交项目主管派活的**方案内容**,不是你动手。
风格:直接、先讲风险和不确定、不讨好。该泼冷水就泼。
产出:结构化咨询方案——目标/范围/【要下游真改东西时·像开发任务包那样圈"写范围(可写目录)+目标文件+工具+验收检查"】/为什么这么判/风险与不确定/必停点/建议的下一步。这份进角色循环交用户审。"#;

// prompt 拼装：静态档案 + ProjectContext（有啥塞啥）+ 用户问题 + 输出格式（要一个 json 块好抠）。
fn consultant_build_prompt(ctx: &ProjectContext, question: &str) -> String {
    let mut p = String::new();
    p.push_str(CONSULTANT_V0_PROFILE);
    p.push_str("\n\n===== 项目上下文（注入·有啥塞啥，不假设齐全）=====\n");
    p.push_str(&format!(
        "项目根: {}\n项目: {}\n最新信号: {}\n",
        ctx.project_root, ctx.project_name, ctx.version_signal
    ));
    if !ctx.document_map.is_empty() {
        p.push_str(&format!(
            "\n--- 文档/结构地图 ---\n{}\n",
            ctx.document_map.join("\n")
        ));
    }
    if ctx.injected_documents.is_empty() {
        p.push_str("\n（未注入任何项目文档正文。）\n");
    } else {
        p.push_str("\n--- 项目文档正文（已注入·你只能依据这些作答）---\n");
        for (path, content) in &ctx.injected_documents {
            p.push_str(&format!("\n### 文件: {path}\n{content}\n"));
        }
    }
    if let Some(bb) = &ctx.blackboard_summary {
        p.push_str(&format!("\n--- 黑板摘要 ---\n{bb}\n"));
    }
    if let Some(mem) = &ctx.memory_summary {
        p.push_str(&format!("\n--- 项目记忆 ---\n{mem}\n"));
    }
    p.push_str(&format!("\n===== 用户的问题 =====\n{question}\n"));
    p.push_str(
        r#"
===== 怎么答 =====
你**读不到**未注入的文件(这个模式没有按需读取工具);**只依据上面已注入的文档正文作答**,不许假设未注入的内容存在。
要交叉核对(如红队 vs 开发计划),就在已注入的对应文档正文里逐条找依据并原文引用。
答完,在最后输出且仅输出一个 ```json 代码块作为结构化产出,严格这个结构:
{
  "user_goal": "用户想达成的",
  "goal_summary": "一句话目标",
  "scope_note": "范围(做什么/不做什么)",
  "reasoning": ["为什么这么判(引用你依据的已注入文档原文)"],
  "risks": [{"severity":"info|warning|blocker","summary":"风险/不确定","mitigation":"怎么缓解"}],
  "must_stop_points": ["必停点"],
  "next_steps": ["建议的下一步"],
  "execution_scope": {
    "target_files": ["预期改动的具体文件·相对项目根(尽量列出·可空)"],
    "checks": ["怎么验收,如 cargo test / npm test / 浏览器打开看效果"]
  },
  "suggest_workflow": true
}
**判断这个目标要不要下游真改代码/文件**:
- **凡用户目标涉及创建/修改/删除任何文件或功能,必须输出 execution_scope**(target_files 按最合理猜测填,宁可猜也别省略);仅当目标是纯提问/纯分析时才给 null。漏给这个字段=用户批的方案会变成不能动手的空转单。
- 要改 → 给出 execution_scope 块,写清"会改哪些文件(target_files)+怎么验收(checks)"。这是你方案的一部分。
  (写范围/工具由系统按固定档位装配·你不用报;多报的字段会被忽略。)
- 只是回答问题、不需要改任何东西 → execution_scope 给 **null**,并在 scope_note 注明"纯咨询/只读"。
**判断这活值不值得拆成多步工作流(suggest_workflow)**:
- 需要多步、有先后依赖、值得先看工序图再动手(复杂改造/多文件协作) → suggest_workflow=**true**。
- 一两步就完、或纯咨询不改东西 → suggest_workflow=**false**(缺省)。"#,
    );
    p
}

#[derive(serde::Deserialize)]
struct ConsultRiskJson {
    #[serde(default)]
    severity: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    mitigation: String,
}

#[derive(serde::Deserialize)]
struct ConsultExecutionScopeJson {
    #[serde(default)]
    write_roots: Vec<String>,
    #[serde(default)]
    target_files: Vec<String>,
    #[serde(default)]
    tools: Vec<String>,
    #[serde(default)]
    checks: Vec<String>,
}

#[derive(serde::Deserialize)]
struct ConsultProposalJson {
    #[serde(default)]
    user_goal: String,
    #[serde(default)]
    goal_summary: String,
    #[serde(default)]
    scope_note: String,
    #[serde(default)]
    reasoning: Vec<String>,
    #[serde(default)]
    risks: Vec<ConsultRiskJson>,
    #[serde(default)]
    must_stop_points: Vec<String>,
    #[serde(default)]
    next_steps: Vec<String>,
    // 向后兼容：旧样本没这块 → None；codex 给 null / 整块缺 / write_roots 全空 → 视作纯咨询(None)。
    #[serde(default)]
    execution_scope: Option<ConsultExecutionScopeJson>,
    // 2.5：向后兼容——旧样本缺此字段 → false（纯咨询/简单活默认不建议工作流）。
    #[serde(default)]
    suggest_workflow: bool,
}

// 从 codex 输出抠最后一个 ```json 块（无围栏则退到首尾大括号）。
fn consultant_extract_json_block(raw: &str) -> Option<String> {
    if let Some(idx) = raw.rfind("```json") {
        let after = &raw[idx + "```json".len()..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_string());
        }
    }
    let first = raw.find('{')?;
    let last = raw.rfind('}')?;
    if last > first {
        Some(raw[first..=last].to_string())
    } else {
        None
    }
}

pub(crate) fn parse_consultation_proposal(raw: &str) -> Result<ConsultationProposal, String> {
    let json = consultant_extract_json_block(raw)
        .ok_or_else(|| "咨询输出里没找到结构化 json 块".to_string())?;
    let dto: ConsultProposalJson =
        serde_json::from_str(&json).map_err(|error| format!("咨询 json 解析失败:{error}"))?;
    if dto.goal_summary.trim().is_empty() && dto.reasoning.is_empty() {
        return Err("咨询产出为空(goal_summary 与 reasoning 都空)".to_string());
    }
    Ok(ConsultationProposal {
        user_goal: dto.user_goal,
        goal_summary: dto.goal_summary,
        scope_note: dto.scope_note,
        reasoning: dto.reasoning,
        risks: dto
            .risks
            .into_iter()
            .map(|r| ConsultationRisk {
                severity: if r.severity.trim().is_empty() {
                    "info".to_string()
                } else {
                    r.severity
                },
                summary: r.summary,
                mitigation: r.mitigation,
            })
            .collect(),
        must_stop_points: dto.must_stop_points,
        next_steps: dto.next_steps,
        // 执行范围·Some/None = 咨询是否给了 execution_scope 块（判定要下游改东西）：给了块 → Some；null / 缺 → None
        // （纯咨询/只读）。交办地基 2.1：写范围来源改为**档位**后，Some 不再取决于 write_roots——咨询报的
        // write_roots/tools 留作向后兼容但下游 map 忽略（用档位）；下游真正用的是 checks/target_files。
        execution_scope: dto.execution_scope.map(|es| ConsultationExecutionScope {
            write_roots: es
                .write_roots
                .into_iter()
                .filter(|root| !root.trim().is_empty())
                .collect(),
            target_files: es.target_files,
            tools: es.tools,
            checks: es.checks,
        }),
        // 2.5：咨询判定的「建议按工作流」（缺省 false）。
        suggest_workflow: dto.suggest_workflow,
    })
}

// ===== CliConsultantAgent（tier-1 impl：codex 自带 loop、自己只读读文档）=====
pub(crate) struct CliConsultantAgent {
    pub(crate) timeout_ms: Option<i64>,
}

impl Default for CliConsultantAgent {
    fn default() -> Self {
        Self {
            // 真咨询要 codex 只读交叉读多篇文档(红队/开发计划)+出结构化 JSON，180s 撞超时被 kill（取不回答案）。
            // 调到 420s 留足只读+推理+JSON 的余量；被咨询项目仍只读（confinement 不变）。
            timeout_ms: Some(420_000),
        }
    }
}

impl ConsultantAgent for CliConsultantAgent {
    fn consult(
        &self,
        ctx: &ProjectContext,
        question: &str,
    ) -> Result<ConsultationProposal, String> {
        let prompt = consultant_build_prompt(ctx, question);
        // 结构性只读：readonly_codex_consult 写死 read-only 沙箱·写盘根空·不走执行闸（codex_local_runner）。
        let raw = codex_local_runner::readonly_codex_consult(
            &ctx.project_root,
            &prompt,
            self.timeout_ms,
        )?;
        parse_consultation_proposal(&raw)
    }
}

// 交办地基 2.1·档位（PROFILE_EDIT_TEST_PROJECT）：编辑类目标的执行范围**写死**——写范围=固定测试项目根、
// 工具=读+写能力、角色=codex-dev+project_director。**不可由请求参数改写**（防"能预览任意项目"滑成"能改任意
// 项目"）。真执行仍 path-lock + 沙箱 + 四护栏兜底。返回 (write_roots, tools, role_ids)。
fn profile_edit_test_project_scope() -> (Vec<String>, Vec<String>, Vec<String>) {
    (
        vec![WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string()],
        vec![
            "read_file".to_string(),
            "write_file".to_string(),
            "apply_patch".to_string(),
        ],
        vec!["codex-dev".to_string(), "project_director".to_string()],
    )
}

// ===== 喂 C1（ConsultationProposal → create_project_consultation_proposal 输入）=====
pub(crate) fn map_consultation_to_c1_input(
    proposal: &ConsultationProposal,
    project_root: &str,
    actor_id: &str,
) -> Result<CreateProjectConsultationProposalInput, String> {
    let head: String = proposal.goal_summary.trim().chars().take(40).collect();
    let title = if head.is_empty() {
        "咨询方案".to_string()
    } else {
        format!("咨询方案：{head}")
    };
    let mut proposed_steps: Vec<String> = proposal
        .reasoning
        .iter()
        .chain(proposal.next_steps.iter())
        .cloned()
        .collect();
    let risks = proposal
        .risks
        .iter()
        .map(|r| ProjectConsultationProposalRisk {
            risk_id: format!(
                "risk:consult:{}",
                crate::utils::hash::short_hash(&r.summary)
            ),
            severity: r.severity.clone(),
            summary: r.summary.clone(),
            mitigation: r.mitigation.clone(),
        })
        .collect();
    let acceptance_criteria = if proposal.next_steps.is_empty() {
        vec![proposal.scope_note.clone()]
    } else {
        proposal.next_steps.clone()
    };
    let stop_conditions = if proposal.must_stop_points.is_empty() {
        vec!["requires_user_confirmation".to_string()]
    } else {
        proposal.must_stop_points.clone()
    };
    // 按 execution_scope 分流：要改东西 → **档位装配**（write/tools/roles 写死·固定测试项目），或纯咨询只读。
    let scope_draft = match &proposal.execution_scope {
        // 有执行范围（交办地基 2.1）：write/tools/roles 从**写死的档位**填（不可参数化·防"预览任意项目"滑成
        // "改任意项目"）；checks 仍用咨询提的、target_files 仍进 proposed_steps。写范围来源换成档位后，原
        // "write_roots 越界拒"护栏对象消失（档位=测试项目根·恒合法），故移除。
        Some(es) => {
            // target_files 不丢：塞进 proposed_steps 最前（让方案卡/主管看到具体文件）。
            if !es.target_files.is_empty() {
                proposed_steps.insert(0, format!("目标文件：{}", es.target_files.join("、")));
            }
            let (profile_write_roots, profile_tools, profile_roles) =
                profile_edit_test_project_scope();
            ProjectConsultationProposalScopeDraft {
                allowed_role_ids: profile_roles, // ← 档位：codex-dev + project_director
                allowed_agent_ids: vec![],
                allowed_read_roots: vec![project_root.to_string()],
                allowed_write_roots: profile_write_roots, // ← 档位：固定测试项目根（写死）
                allowed_tools: profile_tools,             // ← 档位：读+写能力（写死）
                allowed_checks: es.checks.clone(),        // ← 仍用咨询提的
                allowed_task_package_kinds: vec!["task_package".to_string()],
                stop_conditions,
                max_worker_dispatches: None,
                max_runtime_minutes: None,
            }
        }
        // 纯咨询/只读：保持只读·空写范围——这是忠实映射"咨询判定不需要改东西"，**不是**默认兜底缺失的写范围。
        None => ProjectConsultationProposalScopeDraft {
            allowed_role_ids: vec!["project_consultant".to_string()],
            allowed_agent_ids: vec![],
            allowed_read_roots: vec![project_root.to_string()],
            allowed_write_roots: vec![],
            allowed_tools: vec!["read_file".to_string()],
            allowed_checks: vec![],
            allowed_task_package_kinds: vec!["task_package".to_string()],
            stop_conditions,
            max_worker_dispatches: None,
            max_runtime_minutes: None,
        },
    };
    Ok(CreateProjectConsultationProposalInput {
        project_root: project_root.to_string(),
        project_id: None,
        workflow_id: None,
        title,
        user_goal: proposal.user_goal.clone(),
        goal_summary: proposal.goal_summary.clone(),
        proposed_steps,
        scope_draft,
        risks,
        acceptance_criteria,
        created_by_role: ProjectConsultationProposalCreatorRole::ProjectConsultant,
        // 2.5：透传咨询的「建议按工作流」判定（写范围/工具走档位·此标记只影响 UI 图区显隐·不碰授权）。
        suggest_workflow: proposal.suggest_workflow,
        actor_id: actor_id.to_string(),
        expected_store_revision: None,
    })
}

// ===== P2·件 A：接咨询 LM 出方案命令（目标 → AI 真咨询 → 写进方案 store·PendingUserConfirmation）=====
// 收目标 → load_project_context → consultant.consult（真 codex 只读·CliConsultantAgent；stub 测试注假咨询不起 codex）
// → map_consultation_to_c1_input → create_proposal（status=PendingUserConfirmation·**不自动确认**）→ 返回新方案。
// 复用 consult/map/create **本体 0-diff**；咨询结构性只读（readonly_codex_consult·不碰执行闸·不写·不起 worker）。
// 出方案就停：不确认、不边界复核、不让授权生效（人闸不省·principles §4）。
#[derive(serde::Deserialize)]
pub(crate) struct RunProjectConsultationRequest {
    pub(crate) project_root: String,
    pub(crate) goal: String,
    #[serde(default)]
    pub(crate) actor_id: Option<String>,
    // workflow_id 保留（前端请求形）：方案在 prepare 阶段才绑 workflow，本命令出方案不用它（故 allow dead_code）。
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) workflow_id: Option<String>,
}

// 内层（同步·spawn_blocking 里调；可单测·注入 stub 咨询不起 codex）。
fn run_project_consultation_inner(
    path: &std::path::Path,
    consultant: &dyn ConsultantAgent,
    project_root: &str,
    goal: &str,
    actor_id: &str,
) -> Result<ProjectConsultationProposal, String> {
    // 1. 装配 ProjectContext（注入策展文档正文·tier-1）+ 用**手里的真实 path** 召回本项目记忆（刀B·真值不死锚）。
    let mut ctx = load_project_context(project_root)?;
    ctx.memory_summary = recall_project_memory_summary_at(path, project_root);
    // 2. 咨询 LM 出方案（结构性只读·readonly_codex_consult·不碰执行闸）。
    let proposal = consultant.consult(&ctx, goal)?;
    // 3. 映射进 C1 输入（含咨询提的执行范围；写范围越界/空值 → Err 早报）。
    let input = map_consultation_to_c1_input(&proposal, project_root, actor_id)?;
    // 4. 写进方案 store（status=PendingUserConfirmation·**不自动确认**·等用户走方案授权）。
    let write_id = format!("run-project-consultation:{}", unix_timestamp_nanos());
    let output = project_consultation_proposal_store::create_proposal(
        path,
        &input,
        unix_timestamp_ms(),
        &write_id,
    )?;
    Ok(output.proposal)
}

#[tauri::command]
async fn run_project_consultation(
    request: RunProjectConsultationRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectConsultationProposal, String> {
    // path 在 await 前从 state 取（State 不能跨进 'static 闭包）；咨询真 codex 长耗时 → spawn_blocking 不冻 UI。
    let path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let consultant = CliConsultantAgent::default();
        let actor_id = request
            .actor_id
            .clone()
            .unwrap_or_else(|| "project-consultant".to_string());
        run_project_consultation_inner(
            &path,
            &consultant,
            &request.project_root,
            &request.goal,
            &actor_id,
        )
    })
    .await
    .map_err(|error| format!("咨询执行线程异常：{error}"))?
}

// 刀B·记忆召回单测。独特 mod 名（consultant_agent 经 include! 进 crate root，用 `tests` 会撞 crate 根
// mod tests；用独特名既满足「测试进 consultant 自己的 mod、不进 lib.rs」，又不冲突）。
#[cfg(test)]
mod consultant_recall_tests {
    use super::*;
    use serde_json::json;

    fn store_with(records: serde_json::Value) -> FormalMemoryStoreV1 {
        serde_json::from_value(json!({
            "store_version": "formal_memory_store_v1",
            "project_id": null,
            "workflow_id": null,
            "revision": 1,
            "records": records,
            "versions": [],
            "audit_events": [],
            "updated_at": "2026-07-06",
            "warnings": []
        }))
        .expect("fixture store 应可反序列化")
    }

    fn record_json(
        project_id_value: &str,
        memory_type: &str,
        claim: &str,
        body: &str,
        status: &str,
        updated_at: &str,
    ) -> serde_json::Value {
        json!({
            "memory_id": format!("mem:{claim}"),
            "schema_version": "memory_record_v1",
            "record_version": 1,
            "scope": {
                "scope_id": "s",
                "scope_type": "workflow",
                "user_id": null,
                "project_id": project_id_value,
                "workflow_id": null,
                "session_id": null,
                "role_ids": [],
                "document_refs": [],
                "permission_policy_ref": null,
                "model_export_policy": "no_export",
                "valid_from": updated_at,
                "valid_until": null
            },
            "memory_type": memory_type,
            "claim": claim,
            "body": body,
            "source_refs": [],
            "status": status,
            "supersedes_memory_id": null,
            "superseded_by_memory_id": null,
            "conflict_refs": [],
            "audit_refs": [],
            "created_at": updated_at,
            "updated_at": updated_at
        })
    }

    #[test]
    fn recall_active_project_records_ordered_and_filtered() {
        let pid = project_id("/tmp/proj-a");
        let store = store_with(json!([
            record_json(
                &pid,
                "workflow_summary",
                "做了A",
                "证据A\n第二行",
                "memory_active",
                "2026-07-01"
            ),
            record_json(
                &pid,
                "process_fact",
                "做了B",
                "证据B",
                "memory_active",
                "2026-07-02"
            ),
            // 别项目 → 不召回
            record_json(
                "project:other",
                "workflow_summary",
                "别项目记忆",
                "x",
                "memory_active",
                "2026-07-09"
            ),
            // 本项目但非活跃态 → 不召回
            record_json(
                &pid,
                "workflow_summary",
                "已退休记忆",
                "x",
                "memory_deprecated",
                "2026-07-09"
            ),
        ]));
        let summary = recall_from_store(&store, "/tmp/proj-a").expect("有活跃项目记忆 → Some");
        assert!(
            summary.contains("做了A") && summary.contains("做了B"),
            "召回本项目两条活跃记忆：{summary}"
        );
        assert!(!summary.contains("别项目记忆"), "别项目不召回");
        assert!(!summary.contains("已退休记忆"), "非活跃态不召回");
        assert!(
            summary.contains("[workflow_summary] 做了A——证据A"),
            "人话行格式 + body 只取首行"
        );
        assert!(!summary.contains("第二行"), "body 只取首行");
        assert!(summary.lines().count() <= 5, "顶格 5 条");
        assert!(
            summary.find("做了B").unwrap() < summary.find("做了A").unwrap(),
            "更新时间倒序：B(07-02) 在 A(07-01) 前"
        );
    }

    #[test]
    fn recall_none_when_no_active_match() {
        let store = store_with(json!([record_json(
            "project:other",
            "workflow_summary",
            "别项目",
            "x",
            "memory_active",
            "2026-07-01"
        ),]));
        assert!(
            recall_from_store(&store, "/tmp/proj-a").is_none(),
            "本项目 0 条 → None"
        );
        assert!(
            recall_from_store(&store_with(json!([])), "/tmp/proj-a").is_none(),
            "空 store → None"
        );
    }

    #[test]
    fn recall_broken_sidecar_none_not_err() {
        use std::fs;
        let dir = std::env::temp_dir().join(format!("recall-broken-{}", unix_timestamp_string()));
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("workflow-state.v0.json");
        fs::write(&state_path, "{}").unwrap();
        fs::write(dir.join("formal-memories.v1.json"), "{not valid json").unwrap();
        // 坏 sidecar → load_store Err → 静默 None（不 panic、不 Err、不挡咨询）。
        assert!(
            recall_project_memory_summary_at(&state_path, "/tmp/proj-a").is_none(),
            "坏 sidecar → None 不 Err"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 防回潜：load_project_context 恢复纯装配、**不再自带召回**（死锚 default 已挪走）；
    // 召回由各调用方用手里的真实 path 填（consultant / 预拆 / 重拆 三处）。
    #[test]
    fn load_project_context_does_not_self_recall() {
        use std::fs;
        let dir = std::env::temp_dir().join(format!("ctx-no-recall-{}", unix_timestamp_string()));
        fs::create_dir_all(&dir).unwrap();
        let ctx = load_project_context(&dir.to_string_lossy()).expect("装配应成功");
        assert!(
            ctx.memory_summary.is_none(),
            "load_project_context 必须纯装配·不自带召回（防死锚回潜）"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
