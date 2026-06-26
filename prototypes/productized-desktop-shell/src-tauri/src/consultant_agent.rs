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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationProposal {
    pub(crate) user_goal: String,
    pub(crate) goal_summary: String,
    pub(crate) scope_note: String,
    pub(crate) reasoning: Vec<String>,
    pub(crate) risks: Vec<ConsultationRisk>,
    pub(crate) must_stop_points: Vec<String>,
    pub(crate) next_steps: Vec<String>,
}

pub(crate) trait ConsultantAgent {
    fn consult(&self, ctx: &ProjectContext, question: &str) -> Result<ConsultationProposal, String>;
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

// 装配 ProjectContext：有啥塞啥、不假设齐全（防御式降级）。黑板/记忆：被咨询项目通常无工作台数据 → None。
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
        memory_summary: None,
    })
}

// ===== v0 静态档案（spec §3C，写死·先不 derive）=====
const CONSULTANT_V0_PROFILE: &str = r#"你是「项目咨询」。
职责:解答用户对这个项目的问题、帮定大方向。你不主导执行(那是项目主管)、不自己改任何东西。
铁律·落地:每个判断都必须基于你真读到的项目状态。不确定就用工具去读——不许假设某文件/状态/历史存在。结论里引用读到的具体依据。
边界:只读、不写、不执行。给的是建议和方向。若认为该真跑某事,写进方案交项目主管走授权——你自己永不触发执行。
风格:直接、先讲风险和不确定、不讨好。该泼冷水就泼。
产出:结构化咨询方案——目标/范围/为什么这么判/风险与不确定/必停点/建议的下一步。这份进角色循环交用户审。"#;

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
  "next_steps": ["建议的下一步"]
}"#,
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
    fn consult(&self, ctx: &ProjectContext, question: &str) -> Result<ConsultationProposal, String> {
        let prompt = consultant_build_prompt(ctx, question);
        // 结构性只读：readonly_codex_consult 写死 read-only 沙箱·写盘根空·不走执行闸（codex_local_runner）。
        let raw =
            codex_local_runner::readonly_codex_consult(&ctx.project_root, &prompt, self.timeout_ms)?;
        parse_consultation_proposal(&raw)
    }
}

// ===== 喂 C1（ConsultationProposal → create_project_consultation_proposal 输入）=====
pub(crate) fn map_consultation_to_c1_input(
    proposal: &ConsultationProposal,
    project_root: &str,
    actor_id: &str,
) -> CreateProjectConsultationProposalInput {
    let head: String = proposal.goal_summary.trim().chars().take(40).collect();
    let title = if head.is_empty() {
        "咨询方案".to_string()
    } else {
        format!("咨询方案：{head}")
    };
    let proposed_steps: Vec<String> = proposal
        .reasoning
        .iter()
        .chain(proposal.next_steps.iter())
        .cloned()
        .collect();
    let risks = proposal
        .risks
        .iter()
        .map(|r| ProjectConsultationProposalRisk {
            risk_id: format!("risk:consult:{}", crate::utils::hash::short_hash(&r.summary)),
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
    CreateProjectConsultationProposalInput {
        project_root: project_root.to_string(),
        project_id: None,
        workflow_id: None,
        title,
        user_goal: proposal.user_goal.clone(),
        goal_summary: proposal.goal_summary.clone(),
        proposed_steps,
        scope_draft: ProjectConsultationProposalScopeDraft {
            allowed_role_ids: vec!["project_consultant".to_string()],
            allowed_agent_ids: vec![],
            allowed_read_roots: vec![project_root.to_string()],
            allowed_write_roots: vec![], // 咨询只读
            allowed_tools: vec!["read_file".to_string()],
            allowed_checks: vec![],
            allowed_task_package_kinds: vec!["task_package".to_string()],
            stop_conditions,
            max_worker_dispatches: None,
            max_runtime_minutes: None,
        },
        risks,
        acceptance_criteria,
        created_by_role: ProjectConsultationProposalCreatorRole::ProjectConsultant,
        actor_id: actor_id.to_string(),
        expected_store_revision: None,
    }
}
