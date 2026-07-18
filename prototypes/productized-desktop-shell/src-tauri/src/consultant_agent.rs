// S3 agent 智能层·咨询第一刀（v0·tier-1·CLI）。
// 咨询 = 借 codex 的脑 + 本地 harness（v0 静态档案 + ProjectContext 注入 + 只读 confinement）。
// 死线：咨询 codex 结构性只读（codex_local_runner::readonly_codex_consult，read-only 沙箱·写盘根空·
// command_plan_for 不改），**不走 worker 执行闸**（只读 ≠ 执行，但结构性只读挡死写/跑命令）；产出喂 C1。
// 本文件 include! 进 crate root（同 commands.rs/types.rs）：直接用 root 的 C1 类型；std 用全限定避免 use 撞。

// P1-E 诚实关门（2026-07-18 用户拍板 a）：项目主管对话族（说目标出方案/授权后拆任务）目前只开通
// 固定测试项目；非测试项目（含站 3b）统一给这句人话，不再默认走旧塞纸条 fallback。
// consultant_agent.rs/director_agent.rs 经 lib.rs include! 共享同一 crate 根命名空间，两处直接复用。
pub(crate) const HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE: &str =
    "这个项目还没接执行——当前版本先伺候固定测试项目，开放真实项目是后面阶段的事。";

// ===== 契约缝（稳定 trait，下游循环只认它）=====

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationRisk {
    pub(crate) severity: String,
    pub(crate) summary: String,
    pub(crate) mitigation: String,
}

// P2-A：方案自带任务图（终版方案自带任务图·拆任务一跳退场）。字段名逐字对齐 director_agent.rs 的
// `DirectorTaskJson`（title/task_goal/target_role/depends_on/acceptance_criteria/report_format）——
// 别发明第二套字段名（勘察 §2.2 点名）。这是新引入字段，不带历史 `objective` 别名包袱，统一只认 task_goal。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationProposalTask {
    pub(crate) title: String,
    pub(crate) task_goal: String,
    pub(crate) target_role: String,
    pub(crate) depends_on: Vec<String>,
    pub(crate) acceptance_criteria: Vec<String>,
    pub(crate) report_format: Vec<String>,
}

// 执行范围（像开发任务包那样）：咨询在方案里**自己提出**下游执行需要的写范围/目标文件/工具/检查。
// 可空：纯问答咨询不需要下游改任何东西 → None。**后端只忠实透传·绝不默认/兜底缺失的写范围（用户硬约束）。**
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct ConsultationExecutionScope {
    // true = 下游需要改文件；false = 下游只读文件/运行只读检查。旧模型没报该字段时由解析层按 true 兼容。
    pub(crate) requires_write: bool,
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
    pub(crate) worker_acceptance_criteria: Vec<String>,
    pub(crate) control_core_acceptance_criteria: Vec<String>,
    pub(crate) supervisor_acceptance_criteria: Vec<String>,
    // None = 纯咨询/只读·不需要下游改任何东西；Some = 咨询判定要下游真改东西·带执行范围。
    pub(crate) execution_scope: Option<ConsultationExecutionScope>,
    // 交办·刀2 2.5：咨询判「这活是否值得拆成多步工作流」（复杂活 true·简单/纯咨询 false）。UI 用它决定图区出不出现。
    pub(crate) suggest_workflow: bool,
    // P2-A：终版方案自带任务图——你批的就是已质检的任务图，批后拆任务一跳退场。空=纯咨询/execution_scope=null
    // 合法（无需下游执行）；要执行的方案 tasks 是否必须非空由确认时 validate_approved_planned_tasks 兜底判。
    pub(crate) tasks: Vec<ConsultationProposalTask>,
}

// P1-E 旧路退役（2026-07-18 用户拍板 a·诚实关门）：ConsultantAgent trait 随其唯一真实现 CliConsultantAgent
// 一并删除——trait 只剩测试脚手架意义，生产 run_project_consultation 命令从不经它调用（测试项目走
// run_resident_project_consultation_inner，非测试项目诚实关门 Err）。

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
    // 质量债·redo 幂等：本单授权窗内「已完成事实」摘要（口供 did/status/产物文件名·不搬产物本体）。
    // 只由 director **重拆分支**（调用方·真实 path）填；load_project_context 纯装配不填（死锚纪律照 memory_summary）。
    pub(crate) prior_completed_summary: Option<String>,
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
        // 同款死锚纪律：已完成事实只由 director 重拆分支用真实 path 填。
        prior_completed_summary: None,
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
    let answer_rules = r#"
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
  "worker_acceptance_criteria": ["只由 worker 完成的可验证事实；文件名、精确内容、字节数和换行等用户文字约束必须逐字保留"],
  "control_core_acceptance_criteria": ["只由控制核心完成的授权、工作项、唯一派发或账本事实"],
  "supervisor_acceptance_criteria": ["只由主管完成的证据检查、终标或用户报告事实"],
  "execution_scope": {
    "requires_write": true,
    "target_files": ["预期改动的具体文件·相对项目根(尽量列出·可空)"],
    "checks": ["怎么验收,如 cargo test / npm test / 浏览器打开看效果"]
  },
  "suggest_workflow": true,
  "tasks": [
    {
      "title": "任务名",
      "task_goal": "自包含完整指令:做什么 + 目标文件完整路径 + 要写的具体内容 + 依据(worker 只看这段就能干,不引用方案/别任务，不许写"参见方案/见上文/如上所述")",
      "target_role": "执行角色(如 codex-dev)",
      "depends_on": ["前置任务的 title(无前置则空数组)"],
      "acceptance_criteria": ["怎么算这个任务完成"],
      "report_format": ["worker 该结构化返回哪些:做了啥 / 产出在哪 / 成败"]
    }
  ]
}
**判断这个目标要不要下游真改代码/文件**:
- **凡用户目标需要下游读取项目文件、运行检查或创建/修改/删除文件,都必须输出 execution_scope**；仅当目标是无需下游工作的纯问答时才给 null。漏给这个字段=用户批的方案会变成不能执行的空转单。
- 要改 → requires_write=true，写清"会改哪些文件(target_files)+怎么验收(checks)"。这是你方案的一部分。
  (写范围/工具由系统按固定档位装配·你不用报;多报的字段会被忽略。)
- 只读盘点/检查 → requires_write=false，target_files 写只读涉及的文件，checks 只列用户原文明确要求的只读命令；不得自行增加命令。
- 无需读取项目文件或运行检查的纯回答 → execution_scope 给 **null**,并在 scope_note 注明"纯咨询"。
- 三类 acceptance criteria 必须按责任主体输出，不能把 worker、控制核心和主管职责混在同一数组；不要仅概括用户给出的文件名、精确内容、编码、字节数或末尾换行约束，必须逐字保留在 worker_acceptance_criteria。
- **纯咨询/只读/盘点类目标同样三类都不许为空**：worker 的可验证事实=口供本身的硬要求（如"问题清单每条带 file:line+原文引用""对照 README 逐条判定""不写任何文件"），控制核心=零写根/只读沙箱/唯一派发等账本事实，主管=核对口供引用与终标报告。没有文件改动 ≠ 没有 worker 验收。
**判断这活值不值得拆成多步工作流(suggest_workflow)**:
- 需要多步、有先后依赖、值得先看工序图再动手(复杂改造/多文件协作) → suggest_workflow=**true**。
- 一两步就完、或纯咨询不改东西 → suggest_workflow=**false**(缺省)。
**任务图(tasks)——你批的就是已质检的任务图,批后不再重拆**:
- execution_scope 非 null(要下游真跑,不论 requires_write true/false)→ **必须同时给 tasks**：把这个方案拆成
  1 个或多个可直接派发给 worker 的具体任务；哪怕只有一步，也要用 1 个 task 表达(别把 tasks 留空指望批准后
  有人再拆一次)。execution_scope=null(纯咨询/不需下游执行)→ tasks 可以留空。
- tasks 里每个任务的字段规则、worker 工具箱事实(worker 只有 shell，没有独立的 read_file/write_file 等工具名，
  任务文本绝不能禁止 shell 或指定不存在的工具)、task_goal 自包含铁律，与项目主管拆任务时完全同一套标准
  (见下方【worker 工具箱事实】)——这套标准不是摆设，任务文本一旦违反会在方案确认前就被打回重出，不会等
  批准后才发现。
{TOOLBOX_FACTS}"#;
    p.push_str(&answer_rules.replace("{TOOLBOX_FACTS}", DIRECTOR_WORKER_TOOLBOX_FACTS));
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
    requires_write: Option<bool>,
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
struct ConsultTaskJson {
    #[serde(default)]
    title: String,
    #[serde(default)]
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
    #[serde(default)]
    worker_acceptance_criteria: Vec<String>,
    #[serde(default)]
    control_core_acceptance_criteria: Vec<String>,
    #[serde(default)]
    supervisor_acceptance_criteria: Vec<String>,
    // 向后兼容：旧样本没这块 → None；codex 给 null / 整块缺 / write_roots 全空 → 视作纯咨询(None)。
    #[serde(default)]
    execution_scope: Option<ConsultExecutionScopeJson>,
    // 2.5：向后兼容——旧样本缺此字段 → false（纯咨询/简单活默认不建议工作流）。
    #[serde(default)]
    suggest_workflow: bool,
    // P2-A：方案自带任务图；老模型不发 → 空数组（向后兼容，走 director.plan fallback 路）。
    #[serde(default)]
    tasks: Vec<ConsultTaskJson>,
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
        worker_acceptance_criteria: dto.worker_acceptance_criteria,
        control_core_acceptance_criteria: dto.control_core_acceptance_criteria,
        supervisor_acceptance_criteria: dto.supervisor_acceptance_criteria,
        // 执行范围·Some/None = 咨询是否给了 execution_scope 块（判定要下游改东西）：给了块 → Some；null / 缺 → None
        // （纯咨询/只读）。交办地基 2.1：写范围来源改为**档位**后，Some 不再取决于 write_roots——咨询报的
        // write_roots/tools 留作向后兼容但下游 map 忽略（用档位）；下游真正用的是 checks/target_files。
        execution_scope: dto.execution_scope.map(|es| ConsultationExecutionScope {
            // 向后兼容旧结构：此前只要有 execution_scope 就表示要改文件。
            requires_write: es.requires_write.unwrap_or(true),
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
        // P2-A：方案自带任务图，字段名逐字对齐 DirectorTaskJson（别发明第二套字段名）。
        tasks: dto
            .tasks
            .into_iter()
            .map(|t| ConsultationProposalTask {
                title: t.title,
                task_goal: t.task_goal,
                target_role: t.target_role,
                depends_on: t.depends_on,
                acceptance_criteria: t.acceptance_criteria,
                report_format: t.report_format,
            })
            .collect(),
    })
}

// P1-B keeps the legacy consultant proposal parser intact for the CLI path, but
// gives the fixed-project resident conversation one strictly tagged turn shape:
// either a proposal or one supervisor question.  A question is never inferred
// from free text, and a malformed turn is deliberately a conservative stop.
const SUPERVISOR_RESIDENT_TURN_SCHEMA_VERSION: &str = "supervisor_resident_turn.v1";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ResidentSupervisorQuestion {
    pub(crate) question_id: String,
    pub(crate) project_id: String,
    pub(crate) workflow_id: String,
    pub(crate) round: u64,
    pub(crate) question: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResidentQuestionExpectation {
    pub(crate) question_id: String,
    pub(crate) project_id: String,
    pub(crate) workflow_id: String,
    pub(crate) round: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResidentConsultationTurn {
    Proposal(ConsultationProposal),
    SupervisorQuestion(ResidentSupervisorQuestion),
}

fn resident_turn_protocol_error(detail: &str) -> String {
    format!("protocol_invalid:supervisor_resident_turn_{detail}")
}

fn resident_turn_required_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, String> {
    let value = object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| resident_turn_protocol_error(&format!("{field}_missing_or_not_string")))?;
    if value.trim().is_empty() {
        return Err(resident_turn_protocol_error(&format!("{field}_empty")));
    }
    Ok(value.to_string())
}

fn resident_turn_reject_unknown_fields(
    object: &serde_json::Map<String, Value>,
    allowed: &[&str],
) -> Result<(), String> {
    let mut unknown = object
        .keys()
        .filter(|field| !allowed.contains(&field.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unknown.sort();
    if let Some(field) = unknown.first() {
        return Err(resident_turn_protocol_error(&format!(
            "unknown_field:{field}"
        )));
    }
    Ok(())
}

pub(crate) fn parse_resident_consultation_turn(
    raw: &str,
    expected_question: &ResidentQuestionExpectation,
) -> Result<ResidentConsultationTurn, String> {
    // Unlike the legacy CLI consultant, resident turns deliberately do not
    // salvage a fenced block or braces embedded in prose.  The protocol says
    // one bare JSON object; accepting a fragment would weaken the hard gate.
    let json = raw.trim();
    if json.is_empty() {
        return Err(resident_turn_protocol_error("json_object_missing"));
    }
    let value: Value = serde_json::from_str(json)
        .map_err(|error| resident_turn_protocol_error(&format!("json_parse_failed:{error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| resident_turn_protocol_error("not_object"))?;
    let schema_version = resident_turn_required_string(object, "schema_version")?;
    if schema_version != SUPERVISOR_RESIDENT_TURN_SCHEMA_VERSION {
        return Err(resident_turn_protocol_error("schema_version_mismatch"));
    }
    let kind = resident_turn_required_string(object, "kind")?;
    match kind.as_str() {
        "proposal" => {
            resident_turn_reject_unknown_fields(
                object,
                &[
                    "schema_version",
                    "kind",
                    "user_goal",
                    "goal_summary",
                    "scope_note",
                    "reasoning",
                    "risks",
                    "must_stop_points",
                    "next_steps",
                    "worker_acceptance_criteria",
                    "control_core_acceptance_criteria",
                    "supervisor_acceptance_criteria",
                    "execution_scope",
                    "suggest_workflow",
                    // P2-A：方案自带任务图——漏加此白名单项会让带图方案在严格闸即刻 protocol_invalid 保守停
                    // （勘察 §2.2 点名「第一个会咬人的点」）。
                    "tasks",
                ],
            )?;
            parse_consultation_proposal(json).map(ResidentConsultationTurn::Proposal)
        }
        "supervisor_question" => {
            resident_turn_reject_unknown_fields(
                object,
                &[
                    "schema_version",
                    "kind",
                    "question_id",
                    "project_id",
                    "workflow_id",
                    "round",
                    "question",
                ],
            )?;
            let question_id = resident_turn_required_string(object, "question_id")?;
            let project_id = resident_turn_required_string(object, "project_id")?;
            let workflow_id = resident_turn_required_string(object, "workflow_id")?;
            let question = resident_turn_required_string(object, "question")?;
            let round = object
                .get("round")
                .and_then(Value::as_u64)
                .filter(|round| *round > 0)
                .ok_or_else(|| resident_turn_protocol_error("round_missing_or_invalid"))?;
            if question_id != expected_question.question_id
                || project_id != expected_question.project_id
                || workflow_id != expected_question.workflow_id
                || round != expected_question.round
            {
                return Err(resident_turn_protocol_error("question_identity_mismatch"));
            }
            Ok(ResidentConsultationTurn::SupervisorQuestion(
                ResidentSupervisorQuestion {
                    question_id,
                    project_id,
                    workflow_id,
                    round,
                    question,
                },
            ))
        }
        _ => Err(resident_turn_protocol_error("kind_not_allowed")),
    }
}

pub(crate) fn resident_consultation_turn_schema_prompt(
    expected_question: &ResidentQuestionExpectation,
) -> String {
    format!(
        r#"===== 常驻主管回合协议（本节取代上面的 legacy 最终 JSON 形状）=====
本回合只能输出一个 JSON 对象；不得输出自然语言、Markdown、第二个对象或工具调用。schema_version 必须为 "{SUPERVISOR_RESIDENT_TURN_SCHEMA_VERSION}"。

严格二选一，字段不得混用、不得新增：
1) 出方案：kind="proposal"，允许字段只有 schema_version、kind、user_goal、goal_summary、scope_note、reasoning、risks、must_stop_points、next_steps、worker_acceptance_criteria、control_core_acceptance_criteria、supervisor_acceptance_criteria、execution_scope、suggest_workflow、tasks。proposal 的业务字段仍遵守上面既有咨询方案要求。类型也必须严格匹配：reasoning、must_stop_points、next_steps 和三类 acceptance criteria 都是字符串数组；risks 是对象数组且每项都有 severity、summary、mitigation；execution_scope 只能是 JSON null 或对象（若为对象，requires_write 必须是 JSON literal true 或 false）；suggest_workflow 必须是 JSON literal true 或 false，绝不能是数组、字符串或 null；tasks 是对象数组，每项都有 title、task_goal、target_role、depends_on（字符串数组）、acceptance_criteria（字符串数组）、report_format（字符串数组）——execution_scope 非 null 时 tasks 必须非空，execution_scope 为 null（纯咨询）时 tasks 给空数组。纯咨询/只读答复应输出 "execution_scope": null 和 "suggest_workflow": false。
2) 需要用户补充时：kind="supervisor_question"，且必须只含下面字段并逐字回显工作台预发值：
{{
  "schema_version": "{SUPERVISOR_RESIDENT_TURN_SCHEMA_VERSION}",
  "kind": "supervisor_question",
  "question_id": "{question_id}",
  "project_id": "{project_id}",
  "workflow_id": "{workflow_id}",
  "round": {round},
  "question": "需要用户回答的唯一、具体问题"
}}
如果证据不足且需要用户方向，只能走第 2 种；不要猜测用户答复。任何无法严格满足上述形状的输出都会被工作台保守拒绝。"#,
        question_id = expected_question.question_id,
        project_id = expected_question.project_id,
        workflow_id = expected_question.workflow_id,
        round = expected_question.round,
    )
}

fn resident_consultant_build_prompt(
    ctx: &ProjectContext,
    question: &str,
    expected_question: &ResidentQuestionExpectation,
) -> String {
    let mut prompt = consultant_build_prompt(ctx, question);
    prompt.push_str("\n\n");
    prompt.push_str(&resident_consultation_turn_schema_prompt(expected_question));
    prompt
}

// P1-E 旧路退役（2026-07-18 用户拍板 a·诚实关门）：CliConsultantAgent（非测试项目塞纸条 tier-1 impl）
// 整体删除——固定测试项目早已改走项目主管常驻会话（P1-A），非测试项目现在诚实关门（见路由 else 分支）。
// `readonly_codex_consult`/`consultant_build_prompt`/`parse_consultation_proposal` 等共享器官零碰：
// 全局主管两钩点、秘书、resident 常驻路（P1-A/B）仍在消费。

// 编辑类方案的唯一写授权白名单（站 4）：常量单点、不可配置、不可由咨询请求改写。
// 白名单内才给当前项目根这一条写根；其余项目必须降为纯建议只读，防止“能预览任意项目”滑成“能改任意项目”。
const PROFILE_EDIT_WRITE_PROJECT_ROOTS: [&str; 2] = [
    WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
    STATION_4_WRITE_PROJECT_ROOT,
];

// 返回 (write_roots, tools, role_ids)。None 表示该项目没有编辑写授权，调用方必须构造纯建议只读单。
fn profile_edit_test_project_scope(
    project_root: &str,
) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    PROFILE_EDIT_WRITE_PROJECT_ROOTS
        .iter()
        .any(|root| *root == project_root)
        .then(|| {
            (
                vec![project_root.to_string()],
                // worker 唯一执行工具=shell(director prompt 工具箱事实同口径);写域由沙箱锁定。
                // 曾写 read_file/write_file/apply_patch 三个不存在的独立工具名→主管照抄进任务文本
                // →toolbox lint 拦→重拆死循环(07-18 真单实锤),此处必须与 lint 白名单同一世界观。
                vec!["shell(读写·写域由沙箱锁定)".to_string()],
                vec!["codex-dev".to_string(), "project_director".to_string()],
            )
        })
}

// ===== 喂 C1（ConsultationProposal → create_project_consultation_proposal 输入）=====
#[allow(dead_code)]
pub(crate) fn map_consultation_to_c1_input(
    proposal: &ConsultationProposal,
    project_root: &str,
    actor_id: &str,
) -> Result<CreateProjectConsultationProposalInput, String> {
    map_consultation_to_c1_input_with_user_requirement_snapshot(
        proposal,
        project_root,
        actor_id,
        &proposal.user_goal,
    )
}

fn map_consultation_to_c1_input_with_user_requirement_snapshot(
    proposal: &ConsultationProposal,
    project_root: &str,
    actor_id: &str,
    user_requirement_snapshot: &str,
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
    // 按 execution_scope 分流：要改东西只在白名单项目装配写档位；白名单外降为纯建议只读。
    let scope_draft = match &proposal.execution_scope {
        // 有执行范围：写档位只认固定测试根和 mario test 两个精确根；咨询提供的 write_roots/tools 仍无权
        // 改写档位。白名单外不能带着 checks 伪装成可执行只读单，复用纯建议只读机制。
        Some(es) if es.requires_write => {
            if let Some((profile_write_roots, profile_tools, profile_roles)) =
                profile_edit_test_project_scope(project_root)
            {
                // target_files 不丢：塞进 proposed_steps 最前（让方案卡/主管看到具体文件）。
                if !es.target_files.is_empty() {
                    proposed_steps.insert(0, format!("目标文件：{}", es.target_files.join("、")));
                }
                ProjectConsultationProposalScopeDraft {
                    allowed_role_ids: profile_roles,
                    allowed_agent_ids: vec![],
                    allowed_read_roots: vec![project_root.to_string()],
                    allowed_write_roots: profile_write_roots,
                    allowed_tools: profile_tools,
                    allowed_checks: es.checks.clone(),
                    allowed_task_package_kinds: vec!["task_package".to_string()],
                    stop_conditions,
                    max_worker_dispatches: None,
                    max_runtime_minutes: None,
                }
            } else {
                proposed_steps.insert(0, "该项目未获写授权,已降为只读方案".to_string());
                if !es.target_files.is_empty() {
                    proposed_steps.insert(1, format!("目标文件：{}", es.target_files.join("、")));
                }
                ProjectConsultationProposalScopeDraft {
                    allowed_role_ids: vec![
                        "project_consultant".to_string(),
                        "codex-dev".to_string(),
                    ],
                    allowed_agent_ids: vec![],
                    allowed_read_roots: vec![project_root.to_string()],
                    allowed_write_roots: vec![],
                    allowed_tools: vec!["shell(只读: cat/ls/sed)".to_string()],
                    allowed_checks: vec![],
                    allowed_task_package_kinds: vec!["task_package".to_string()],
                    stop_conditions,
                    max_worker_dispatches: None,
                    max_runtime_minutes: None,
                }
            }
        }
        // 只读执行范围：允许读项目文件与运行用户明确要求的检查，但写根保持空。
        Some(es) => {
            if !es.target_files.is_empty() {
                proposed_steps.insert(0, format!("只读文件：{}", es.target_files.join("、")));
            }
            for check in &es.checks {
                if !user_requirement_snapshot.contains(check) {
                    return Err(format!(
                        "consultant_readonly_check_not_in_user_requirement:{check}"
                    ));
                }
            }
            ProjectConsultationProposalScopeDraft {
                allowed_role_ids: vec!["project_consultant".to_string(), "codex-dev".to_string()],
                allowed_agent_ids: vec![],
                allowed_read_roots: vec![project_root.to_string()],
                allowed_write_roots: vec![],
                allowed_tools: vec!["shell(只读: cat/ls/sed)".to_string()],
                allowed_checks: es.checks.clone(),
                allowed_task_package_kinds: vec!["task_package".to_string()],
                stop_conditions,
                max_worker_dispatches: None,
                max_runtime_minutes: None,
            }
        }
        // 纯咨询：保持只读·空写范围——这是忠实映射"咨询判定无需下游执行"，**不是**默认兜底缺失的写范围。
        None => ProjectConsultationProposalScopeDraft {
            // 只读结论仍需交给 worker 执行；仅授权 codex-dev 角色，不授予任何写范围或写工具。
            allowed_role_ids: vec!["project_consultant".to_string(), "codex-dev".to_string()],
            allowed_agent_ids: vec![],
            allowed_read_roots: vec![project_root.to_string()],
            allowed_write_roots: vec![],
            allowed_tools: vec!["shell(只读: cat/ls/sed)".to_string()],
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
        user_requirement_snapshot: user_requirement_snapshot.to_string(),
        goal_summary: proposal.goal_summary.clone(),
        proposed_steps,
        scope_draft,
        risks,
        worker_acceptance_criteria: proposal.worker_acceptance_criteria.clone(),
        control_core_acceptance_criteria: proposal.control_core_acceptance_criteria.clone(),
        supervisor_acceptance_criteria: proposal.supervisor_acceptance_criteria.clone(),
        acceptance_criteria,
        created_by_role: ProjectConsultationProposalCreatorRole::ProjectConsultant,
        // 2.5：透传咨询的「建议按工作流」判定（写范围/工具走档位·此标记只影响 UI 图区显隐·不碰授权）。
        suggest_workflow: proposal.suggest_workflow,
        // P2-A：方案自带任务图透传（字段名逐字对齐，见 ConsultationProposalTask 定义处注释）。
        tasks: proposal
            .tasks
            .iter()
            .map(|t| ProjectConsultationProposalTask {
                title: t.title.clone(),
                task_goal: t.task_goal.clone(),
                target_role: t.target_role.clone(),
                depends_on: t.depends_on.clone(),
                acceptance_criteria: t.acceptance_criteria.clone(),
                report_format: t.report_format.clone(),
            })
            .collect(),
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
    // P1-B fixed-project resident turns need the server-bound project/workflow
    // identity for a strict supervisor-question envelope.  Other legacy
    // read-only consultant callers may still omit it.
    #[serde(default)]
    pub(crate) project_id: Option<String>,
    #[serde(default)]
    pub(crate) workflow_id: Option<String>,
}

// P2-A：出方案/确认答复共用的落库前闸——lint 前移到这里，方案带毒任务图（禁 shell/引用不存在工具/
// 误把注入材料当唯一事实来源）在批之前就被拒，用户看不到「批准后拆任务死循环」。两个方案产出口
// （首问 run_resident_project_consultation_inner / 问答后 submit_supervisor_resident_answer_with）都经
// 同一个 write_consultation_proposal 落库，挂这里=一处覆盖两口（勘察 §2.1「两个方案产出口都经同一写漏斗」）。
// 保守停不自动重出：与 resident 侧其它 protocol 错误同款处置——上抛错误，用户在对话里再说一次即可重新出方案。
fn lint_consultation_proposal_tasks(tasks: &[ConsultationProposalTask]) -> Result<(), String> {
    if let Some((task, reason)) = tasks.iter().find_map(|task| {
        worker_toolbox_lint_reason_for_text(
            &task.title,
            &task.task_goal,
            &task.acceptance_criteria,
            &task.report_format,
        )
        .map(|reason| (task, reason))
    }) {
        return Err(format!(
            "proposal_task_toolbox_lint_failed:任务「{}」{}；方案已拒绝，请重新说一次目标出方案。",
            task.title, reason
        ));
    }
    Ok(())
}

pub(crate) fn write_consultation_proposal(
    path: &std::path::Path,
    proposal: &ConsultationProposal,
    project_root: &str,
    goal: &str,
    actor_id: &str,
) -> Result<ProjectConsultationProposal, String> {
    // P2-A：方案带任务图 → 出方案就 lint，批的就是已质检的图（在 map/create 之前拦，方案根本不会落库）。
    lint_consultation_proposal_tasks(&proposal.tasks)?;
    // 映射进 C1 输入（含咨询提的执行范围；写范围越界/空值 → Err 早报）。
    let input = map_consultation_to_c1_input_with_user_requirement_snapshot(
        proposal,
        project_root,
        actor_id,
        goal,
    )?;
    // 写进方案 store（status=PendingUserConfirmation·**不自动确认**·等用户走方案授权）。
    let write_id = format!("run-project-consultation:{}", unix_timestamp_nanos());
    let output = project_consultation_proposal_store::create_proposal(
        path,
        &input,
        unix_timestamp_ms(),
        &write_id,
    )?;
    Ok(output.proposal)
}

// P1-E 旧路退役：run_project_consultation_inner（内层·注入 ConsultantAgent trait 对象）随 trait 一并删除——
// 已无生产调用者。它保护的「出方案即停·PendingUserConfirmation·不自动建授权」不变量改由测试直喂
// write_consultation_proposal（本函数原先的下游共享落点，resident 成功分支也走它）覆盖。

fn run_resident_project_consultation_inner(
    path: &std::path::Path,
    project_root: &str,
    workflow_id: &str,
    goal: &str,
    actor_id: &str,
) -> Result<ProjectConsultationProposal, String> {
    if workflow_id.trim().is_empty() {
        return Err("supervisor_resident_workflow_id_required".to_string());
    }
    // A resident conversation can have at most one canonical question waiting
    // for a user.  Share the answer-command lock so two opening turns cannot
    // pre-issue the same question identity before either one persists it.
    let _guard = supervisor_session_launcher::resident_conversation_lock()
        .lock()
        .map_err(|_| "supervisor_resident_conversation_lock_poisoned".to_string())?;
    let mut ctx = load_project_context(project_root)?;
    ctx.memory_summary = recall_project_memory_summary_at(path, project_root);
    let expected_question = supervisor_session_launcher::next_resident_question_expectation(
        path,
        project_root,
        workflow_id,
    )?;
    let prompt = resident_consultant_build_prompt(&ctx, goal, &expected_question);
    let turn = supervisor_session_launcher::consult_supervisor_resident_turn(
        path,
        project_root,
        workflow_id,
        &prompt,
        "project_consult",
    )?;
    match parse_resident_consultation_turn(&turn.content, &expected_question)? {
        ResidentConsultationTurn::Proposal(proposal) => {
            write_consultation_proposal(path, &proposal, project_root, goal, actor_id)
        }
        ResidentConsultationTurn::SupervisorQuestion(question) => {
            supervisor_session_launcher::record_supervisor_resident_question_asked(
                path,
                project_root,
                &question,
                goal,
                &turn.thread_id,
            )?;
            Err(format!(
                "supervisor_resident_question_waiting_user:{}:第{}轮主管问题已写入项目黑板，等待用户答复。",
                question.question_id, question.round
            ))
        }
    }
}

#[tauri::command]
async fn run_project_consultation(
    request: RunProjectConsultationRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectConsultationProposal, String> {
    // path 在 await 前从 state 取（State 不能跨进 'static 闭包）；咨询真 codex 长耗时 → spawn_blocking 不冻 UI。
    let path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let actor_id = request
            .actor_id
            .clone()
            .unwrap_or_else(|| "project-consultant".to_string());
        if request.project_root == WORKFLOW_ENGINE_TEST_PROJECT_ROOT {
            let derived_project_id = project_id(&request.project_root);
            if request
                .project_id
                .as_deref()
                .is_some_and(|project_id| project_id != derived_project_id)
            {
                return Err("supervisor_resident_project_id_mismatch".to_string());
            }
            let workflow_id = request
                .workflow_id
                .as_deref()
                .filter(|workflow_id| !workflow_id.trim().is_empty())
                .ok_or_else(|| "supervisor_resident_workflow_id_required".to_string())?;
            run_resident_project_consultation_inner(
                &path,
                &request.project_root,
                workflow_id,
                &request.goal,
                &actor_id,
            )
        } else {
            // P1-E 诚实关门（用户 07-18 拍板 a）：项目主管对话族只开通固定测试项目；旧「非测试项目
            // 塞纸条 fallback」（CliConsultantAgent）整体退役，不豁免站 3b（用户拍板明确扩到 3b）。
            Err(HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE.to_string())
        }
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
        assert!(
            ctx.prior_completed_summary.is_none(),
            "load_project_context 不填已完成事实——由 director 重拆分支用真实 path 填（防死锚回潜·同款）"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn edit_scope_whitelist_keeps_only_two_exact_roots_and_downgrades_all_others() {
        let proposal = ConsultationProposal {
            user_goal: "创建单个文件".to_string(),
            goal_summary: "需要写入一个目标文件".to_string(),
            scope_note: "编辑方案".to_string(),
            reasoning: vec!["先核对范围".to_string()],
            risks: vec![],
            must_stop_points: vec![],
            next_steps: vec!["执行已批准方案".to_string()],
            worker_acceptance_criteria: vec!["返回写入证据".to_string()],
            control_core_acceptance_criteria: vec!["核对授权范围".to_string()],
            supervisor_acceptance_criteria: vec!["复核结果".to_string()],
            execution_scope: Some(ConsultationExecutionScope {
                requires_write: true,
                write_roots: vec!["/etc".to_string()],
                target_files: vec!["test.txt".to_string()],
                tools: vec!["shell".to_string()],
                checks: vec!["cargo test --lib".to_string()],
            }),
            suggest_workflow: false,
            tasks: vec![],
        };

        for project_root in [
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            STATION_4_WRITE_PROJECT_ROOT,
        ] {
            let input = map_consultation_to_c1_input(&proposal, project_root, "tester")
                .expect("白名单项目应保留编辑档位");
            assert_eq!(
                input.scope_draft.allowed_write_roots,
                vec![project_root.to_string()]
            );
            assert_eq!(
                input.scope_draft.allowed_tools,
                vec!["shell(读写·写域由沙箱锁定)".to_string()]
            );
            assert_eq!(
                input.scope_draft.allowed_checks,
                vec!["cargo test --lib".to_string()]
            );
        }

        for project_root in [
            "/Users/yoyi/gameai/crazytown".to_string(),
            format!("{STATION_4_WRITE_PROJECT_ROOT}/"),
            format!("{STATION_4_WRITE_PROJECT_ROOT}/subdir"),
        ] {
            let input = map_consultation_to_c1_input(&proposal, &project_root, "tester")
                .expect("白名单外编辑意图应降为纯建议只读单");
            assert!(input.scope_draft.allowed_write_roots.is_empty());
            assert_eq!(
                input.scope_draft.allowed_tools,
                vec!["shell(只读: cat/ls/sed)".to_string()]
            );
            assert!(input.scope_draft.allowed_checks.is_empty());
            assert_eq!(
                input.proposed_steps.first().map(String::as_str),
                Some("该项目未获写授权,已降为只读方案")
            );
        }
    }

    #[test]
    fn readonly_execution_scope_carries_user_literal_check_without_write_authority() {
        let raw = r#"```json
{
  "user_goal": "只读盘点",
  "goal_summary": "核对源码并检查语法",
  "scope_note": "只读",
  "reasoning": ["只读检查"],
  "risks": [],
  "must_stop_points": ["不得写文件"],
  "next_steps": ["交给只读 worker"],
  "worker_acceptance_criteria": ["运行 node --check game.js"],
  "control_core_acceptance_criteria": ["零写根"],
  "supervisor_acceptance_criteria": ["核对退出码"],
  "execution_scope": {
    "requires_write": false,
    "target_files": ["README.md", "game.js"],
    "checks": ["node --check game.js"]
  },
  "suggest_workflow": false
}
```"#;
        let proposal = parse_consultation_proposal(raw).expect("parse readonly scope");
        let input = map_consultation_to_c1_input_with_user_requirement_snapshot(
            &proposal,
            "/tmp/mario-test",
            "user",
            "只读读取 README.md 和 game.js；运行 node --check game.js；不写文件。",
        )
        .expect("literal user check should map");

        assert!(input.scope_draft.allowed_write_roots.is_empty());
        assert_eq!(input.scope_draft.allowed_tools, vec!["shell(只读: cat/ls/sed)"]);
        assert_eq!(
            input.scope_draft.allowed_checks,
            vec!["node --check game.js"]
        );
        assert_eq!(
            input.proposed_steps.first().map(String::as_str),
            Some("只读文件：README.md、game.js")
        );
    }

    #[test]
    fn readonly_execution_scope_rejects_model_invented_check() {
        let proposal = ConsultationProposal {
            user_goal: "只读盘点".to_string(),
            goal_summary: "核对源码".to_string(),
            scope_note: "只读".to_string(),
            reasoning: vec!["只读检查".to_string()],
            risks: vec![],
            must_stop_points: vec!["不得写文件".to_string()],
            next_steps: vec!["交给只读 worker".to_string()],
            worker_acceptance_criteria: vec!["运行 npm test".to_string()],
            control_core_acceptance_criteria: vec!["零写根".to_string()],
            supervisor_acceptance_criteria: vec!["核对退出码".to_string()],
            execution_scope: Some(ConsultationExecutionScope {
                requires_write: false,
                write_roots: vec![],
                target_files: vec!["game.js".to_string()],
                tools: vec![],
                checks: vec!["npm test".to_string()],
            }),
            suggest_workflow: false,
            tasks: vec![],
        };
        let error = map_consultation_to_c1_input_with_user_requirement_snapshot(
            &proposal,
            "/tmp/mario-test",
            "user",
            "只读盘点 game.js，不写文件。",
        )
        .expect_err("model must not invent readonly checks");
        assert_eq!(
            error,
            "consultant_readonly_check_not_in_user_requirement:npm test"
        );
    }
}
