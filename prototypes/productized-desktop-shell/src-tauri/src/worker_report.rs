// worker 回程契约：报文从「嘱咐」变「契约」。
//
// 任务包：tasks/2026-07-06-worker-report-contract-backend-v1.md
//
// 本模块只定义「契约 + 解析 + 链消费的可测核心」：
//   - 契约文本（确定性追加给 worker，不经 LM）；
//   - 从 worker 最后消息抠出 json 块并解析（软着陆：抠不到/坏 json → None，不 Err）；
//   - 链每任务完成后消费一次：解析 → 组登记入参 → best-effort 调现成登记机器落库 → 出摘要/求助信号。
//
// 安全属性（安全死线）：完成汇报仍只归档不驱动；求助只暴露强信号，由链调用方停在
// waiting_decision。落库走现成 `record_worker_structured_report_at`（自带校验），best-effort。

use serde::Deserialize;
use std::path::Path;

/// worker 回程契约结构：做了啥 / 产出在哪（路径列表）/ 成败 / 怎么证明。
/// 全 `#[serde(default)]`——缺字段不报错，配合软着陆语义。
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct WorkerReport {
    #[serde(default)]
    pub(crate) did: String,
    #[serde(default)]
    pub(crate) outputs: Vec<String>,
    #[serde(default)]
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) evidence: Vec<String>,
    #[serde(default)]
    pub(crate) permission_requests: Vec<String>,
    #[serde(default)]
    pub(crate) open_issues: Vec<String>,
    #[serde(default)]
    pub(crate) direction_risks: Vec<String>,
    #[serde(default)]
    pub(crate) follow_up_suggestions: Vec<String>,
}

/// 追加给 worker 的契约段（确定性文本·不经 LM·同 consultant/director 的 json 块成熟套路）。
pub(crate) const WORKER_REPORT_CONTRACT_TEXT: &str = r#"回程契约（务必遵守）：干完后，最后输出**且仅输出**一个 ```json 代码块。`did`、`outputs`、`status`、`evidence` 和全部求助字段都只能位于 JSON 顶层；不得嵌套在 `target` 或其他对象中。outputs 写产出文件的完整路径；没有产出就写空数组 []。完成路只使用 done|partial|failed；被阻塞、需要更多权限或资料、或认为方向可能错时，status 必须为 blocked。

完成 done 的完整示例：
```json
{
  "did": "创建目标文件并完成回读和字节验证",
  "outputs": ["/绝对路径/目标文件.txt"],
  "status": "done",
  "evidence": ["回读输出与字节校验命令结果"],
  "permission_requests": [],
  "open_issues": [],
  "direction_risks": [],
  "follow_up_suggestions": []
}
```

受阻 blocked 的完整示例：
```json
{
  "did": "尝试创建目标文件但被授权边界阻止",
  "outputs": [],
  "status": "blocked",
  "evidence": ["写入命令返回的拒绝信息"],
  "permission_requests": ["需要目标目录的写入授权"],
  "open_issues": ["当前 allowed_write 不含目标目录"],
  "direction_risks": ["继续写入会越过已批准范围"],
  "follow_up_suggestions": ["请主管请求用户决定是否扩展授权"]
}
```

实际回程只能保留对应的一份 JSON 代码块，代码块之后不得再写任何字。"#;

/// 物化时给任务包 artifact 的 goals 追加契约：objective 首位 + 主管拆的 report_format 各项 + 契约文本。
/// 确定性拼接·不经 LM（安全死线：契约段不给 LM 发挥空间）。
pub(crate) fn build_goals_with_contract(objective: &str, report_format: &[String]) -> Vec<String> {
    let mut goals = vec![objective.to_string()];
    goals.extend(report_format.iter().cloned());
    goals.push(WORKER_REPORT_CONTRACT_TEXT.to_string());
    goals
}

/// 从 worker 最后消息抠出并解析契约 json 块。
/// 复用 crate-root 现成 `consultant_extract_json_block` 抠块 + serde 解析；
/// 抠不到 / 解析失败 → `None`（**不 Err**——软着陆语义留给调用方）。
pub(crate) fn parse_worker_report(raw: &str) -> Option<WorkerReport> {
    let block = crate::consultant_extract_json_block(raw)?;
    serde_json::from_str::<WorkerReport>(&block).ok()
}

pub(crate) fn help_signal_from_raw(raw: &str) -> Option<WorkerReportHelpSignal> {
    match parse_worker_report(raw) {
        Some(report) => {
            let summary = worker_report_summary(&report);
            worker_report_help_signal(&report, &summary)
        }
        None => suspected_help_signal(raw),
    }
}

/// 链消费一次 worker 报文的结果：都放 **step 级**（不进链级 outcome.warnings）。
pub(crate) struct WorkerReportConsumeOutcome {
    /// did（status）一句话摘要；无契约报文时 None。
    pub(crate) report_summary: Option<String>,
    /// 每任务级报文诊断（落库失败 / 有输出却没按契约）；无诊断时 None。
    /// **放 step 级、不进链级 outcome.warnings**——链级只留结构警告（dangling 依赖等），
    /// worker 报文是每任务的内容层诊断，混入会污染链级语义、也会惊动既有 fake 链测试。
    pub(crate) report_warning: Option<String>,
    /// 刀A·口供上脸：worker 自报 status（done|partial|failed 原值）；status 空/没交口供 → None。
    /// 前端据此判黄牌（呈现不驱动·黄牌不是闸）。
    pub(crate) report_status: Option<String>,
    /// C3a·worker 求助强信号：blocked 或求助字段非空时返回；调用方据此停在 waiting_decision。
    pub(crate) help_signal: Option<WorkerReportHelpSignal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkerReportHelpSignal {
    pub(crate) status: String,
    pub(crate) summary: String,
    pub(crate) open_issues: Vec<String>,
    pub(crate) permission_requests: Vec<String>,
    pub(crate) direction_risks: Vec<String>,
    pub(crate) follow_up_suggestions: Vec<String>,
}

/// 从口供 status 归一化出 step.report_status：trim 后空 → None，否则 Some(原值)。
fn report_status_field(status: &str) -> Option<String> {
    let trimmed = status.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 链每任务**完成后**消费一次 worker 最后消息（全文）：解析 → best-effort 落库（现成登记机器·
/// 自带校验）→ 出摘要或求助强信号。
/// - Some(report) → 组登记入参调 `record_worker_structured_report_at`；落库失败仅出 warning、不断链；
///   summary = did（status）。
/// - None（无块/坏 json）→ warning（附原文尾 200 字）+ summary None；任务仍算完成（软着陆）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn consume_worker_report_after_completion(
    state_path: &Path,
    project_root: &str,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    actor_role: &str,
    task_title: &str,
    last_message_full: &str,
) -> WorkerReportConsumeOutcome {
    match parse_worker_report(last_message_full) {
        Some(report) => {
            let summary = worker_report_summary(&report);
            let help_signal = worker_report_help_signal(&report, &summary);
            let input = build_report_input(
                project_root,
                project_id,
                workflow_id,
                workflow_node_id,
                work_item_id,
                dispatch_id,
                actor_role,
                &report,
            );
            match crate::record_worker_structured_report_at(state_path, &input) {
                Ok(_) => WorkerReportConsumeOutcome {
                    report_summary: if help_signal.is_some() {
                        None
                    } else {
                        Some(summary.clone())
                    },
                    report_warning: None,
                    report_status: report_status_field(&report.status),
                    help_signal,
                },
                Err(err) => WorkerReportConsumeOutcome {
                    report_summary: if help_signal.is_some() {
                        None
                    } else {
                        Some(summary.clone())
                    },
                    report_warning: Some(format!(
                        "任务「{task_title}」报文落库失败（不影响任务完成）：{err}"
                    )),
                    report_status: report_status_field(&report.status),
                    help_signal,
                },
            }
        }
        None => {
            if let Some(help_signal) = suspected_help_signal(last_message_full) {
                return WorkerReportConsumeOutcome {
                    report_summary: None,
                    report_warning: None,
                    report_status: None,
                    help_signal: Some(help_signal),
                };
            }
            // 有内容但抠不到契约块 → worker 没守契约，出一条诊断；last_message 为空（无输出/非真跑）→ 无从判断，静默。
            // 两种情形任务都恒算完成（只归档不驱动）。
            let report_warning = if last_message_full.trim().is_empty() {
                None
            } else {
                Some(format!(
                    "任务「{task_title}」已完成但未按契约交报文（原文尾：{}）",
                    tail_chars(last_message_full, 200)
                ))
            };
            WorkerReportConsumeOutcome {
                report_summary: None,
                report_warning,
                report_status: None,
                help_signal: None,
            }
        }
    }
}

fn worker_report_has_help_signal(report: &WorkerReport) -> bool {
    report.status.trim().eq_ignore_ascii_case("blocked")
        || !report.permission_requests.is_empty()
        || !report.open_issues.is_empty()
        || !report.direction_risks.is_empty()
        || !report.follow_up_suggestions.is_empty()
}

fn worker_report_help_signal(
    report: &WorkerReport,
    summary: &str,
) -> Option<WorkerReportHelpSignal> {
    if !worker_report_has_help_signal(report) {
        return None;
    }
    let status = if report.status.trim().is_empty() {
        "blocked".to_string()
    } else {
        report.status.trim().to_string()
    };
    Some(WorkerReportHelpSignal {
        status,
        summary: summary.to_string(),
        open_issues: report.open_issues.clone(),
        permission_requests: report.permission_requests.clone(),
        direction_risks: report.direction_risks.clone(),
        follow_up_suggestions: report.follow_up_suggestions.clone(),
    })
}

fn suspected_help_signal(raw: &str) -> Option<WorkerReportHelpSignal> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }
    let lowered = text.to_lowercase();
    let markers = [
        "blocked",
        "blocker",
        "need permission",
        "permission denied",
        "permission",
        "stuck",
        "求助",
        "卡住",
        "需要权限",
        "缺权限",
        "权限不足",
        "资料不足",
        "缺资料",
        "无法继续",
        "方向错误",
        "方向可能错",
        "方向不对",
    ];
    if !markers.iter().any(|marker| lowered.contains(marker)) {
        return None;
    }
    let tail = tail_chars(text, 200);
    Some(WorkerReportHelpSignal {
        status: "suspected_blocked".to_string(),
        summary: format!("疑似求助·主管必看（原文尾：{tail}）"),
        open_issues: vec![format!("疑似求助原文尾：{tail}")],
        permission_requests: Vec::new(),
        direction_risks: Vec::new(),
        follow_up_suggestions: vec!["请项目主管判断是否补充权限、资料或调整方向。".to_string()],
    })
}

/// 链步骤摘要：一句话「did（status）」，缺则占位。serde 加法，前端渐进接。
fn worker_report_summary(report: &WorkerReport) -> String {
    let did = report.did.trim();
    let did = if did.is_empty() {
        "（未说明）"
    } else {
        did
    };
    let status = report.status.trim();
    let status = if status.is_empty() { "unknown" } else { status };
    format!("{did}（{status}）")
}

/// WorkerReport → 现成登记入参映射。必填字段兜底非空（登记机器 validate 硬要求）。
/// 字段填法照 `project_workflow_automation.rs` 的现成范本（source_kind/sensitive_level 等）。
#[allow(clippy::too_many_arguments)]
fn build_report_input(
    project_root: &str,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    actor_role: &str,
    report: &WorkerReport,
) -> crate::WorkerStructuredReportInput {
    let timestamp = crate::unix_timestamp_string();
    let did = report.did.trim();
    let status = report.status.trim();
    // 必填 String 字段非空兜底（老实填、缺的用报文原文/占位兜）。
    let executed_what = if did.is_empty() {
        "worker 未在 did 说明做了什么".to_string()
    } else {
        did.to_string()
    };
    let changed_what = if report.outputs.is_empty() {
        "worker 未列出产出文件".to_string()
    } else {
        report.outputs.join("；")
    };
    let summary = if did.is_empty() {
        "worker 报文 did 为空".to_string()
    } else {
        did.to_string()
    };
    // 契约 status（done|partial|failed|blocked）→ 登记机器 acceptance_status 白名单
    // （reported_completed|reported_not_completed|blocked|needs_rework）。
    // 空/未知保守映射为 reported_not_completed（不谎报完成）。
    let acceptance_status = if worker_report_has_help_signal(report) {
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
    // evidence_refs 必须非空（validate 硬要求）：报文 evidence 空则兜一条指向最后消息。
    let evidence_refs = if report.evidence.is_empty() {
        vec!["（worker 未附证据，见 worker 最后消息 json 块）".to_string()]
    } else {
        report.evidence.clone()
    };
    crate::WorkerStructuredReportInput {
        project_root: project_root.to_string(),
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_node_id: workflow_node_id.to_string(),
        work_item_id: work_item_id.to_string(),
        dispatch_id: dispatch_id.map(str::to_string),
        actor_role: actor_role.to_string(),
        executed_what,
        changed_what,
        summary,
        evidence_refs,
        open_issues: report.open_issues.clone(),
        permission_requests: report.permission_requests.clone(),
        direction_risks: report.direction_risks.clone(),
        follow_up_suggestions: report.follow_up_suggestions.clone(),
        acceptance_status,
        // source_refs 必须非空（validate 硬要求）：一条指向 worker 最后消息/派发的来源引用。
        source_refs: vec![crate::ObservationSourceRef {
            source_ref_id: format!("worker-report-src:{work_item_id}:{timestamp}"),
            source_kind: "worker_report".to_string(),
            source_id: dispatch_id.unwrap_or(work_item_id).to_string(),
            project_id: Some(project_id.to_string()),
            workflow_id: Some(workflow_id.to_string()),
            session_id: None,
            file_path: None,
            evidence_ref: Some(format!("work-item:{work_item_id}")),
            summary: "worker 回程契约报文（最后消息 json 块）。".to_string(),
            sensitive_level: "internal".to_string(),
            created_at: timestamp.clone(),
        }],
        expected_workflow_revision: None,
    }
}

/// 取字符串最后 n 个字符（按 char 边界安全），供软着陆时附原文尾。
fn tail_chars(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.trim().chars().collect();
    let start = chars.len().saturating_sub(n);
    chars[start..].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_block() {
        let raw = "干完了。\n```json\n{\"did\":\"改了登录页\",\"outputs\":[\"/p/login.tsx\"],\"status\":\"done\",\"evidence\":[\"cargo test 绿\"]}\n```";
        let report = parse_worker_report(raw).expect("合法块应解析");
        assert_eq!(report.did, "改了登录页");
        assert_eq!(report.outputs, vec!["/p/login.tsx"]);
        assert_eq!(report.status, "done");
        assert_eq!(report.evidence, vec!["cargo test 绿"]);
    }

    #[test]
    fn missing_fields_default() {
        // 只给 did/status，缺 outputs/evidence → default 空数组（不报错）。
        let raw = "```json\n{\"did\":\"只做了一半\",\"status\":\"partial\"}\n```";
        let report = parse_worker_report(raw).expect("缺字段也应解析");
        assert_eq!(report.did, "只做了一半");
        assert_eq!(report.status, "partial");
        assert!(report.outputs.is_empty());
        assert!(report.evidence.is_empty());
    }

    #[test]
    fn parses_blocked_help_fields_with_defaults() {
        let legacy = parse_worker_report(
            "```json\n{\"did\":\"旧报文\",\"outputs\":[],\"status\":\"done\",\"evidence\":[]}\n```",
        )
        .expect("旧报文缺求助字段也应解析");
        assert!(legacy.permission_requests.is_empty());
        assert!(legacy.open_issues.is_empty());
        assert!(legacy.direction_risks.is_empty());
        assert!(legacy.follow_up_suggestions.is_empty());

        let raw = "```json\n{\"did\":\"卡住\",\"outputs\":[],\"status\":\"blocked\",\"evidence\":[\"缺权限\"],\"permission_requests\":[\"需要读取 /secure\"],\"open_issues\":[\"缺验收数据\"],\"direction_risks\":[\"当前方向可能会改错文件\"],\"follow_up_suggestions\":[\"请主管补充目标文件\"]}\n```";
        let report = parse_worker_report(raw).expect("blocked 求助报文应解析");
        assert_eq!(report.status, "blocked");
        assert_eq!(report.permission_requests, vec!["需要读取 /secure"]);
        assert_eq!(report.open_issues, vec!["缺验收数据"]);
        assert_eq!(report.direction_risks, vec!["当前方向可能会改错文件"]);
        assert_eq!(report.follow_up_suggestions, vec!["请主管补充目标文件"]);
    }

    #[test]
    fn no_block_is_none() {
        assert!(parse_worker_report("我做完了但没给 json").is_none());
        assert!(parse_worker_report("").is_none());
    }

    #[test]
    fn broken_json_is_none() {
        // 有块但 json 坏 → None（软着陆，不 panic、不 Err）。
        let raw = "```json\n{\"did\":\"漏了引号,status:done}\n```";
        assert!(parse_worker_report(raw).is_none());
    }

    #[test]
    fn prose_around_block_ok() {
        // 块前后有废话 → 照抠（复用抠取器行为）。
        let raw = "先说一堆背景。\n\n```json\n{\"did\":\"d\",\"outputs\":[],\"status\":\"done\",\"evidence\":[]}\n```\n\n（本不该有的后记）";
        let report = parse_worker_report(raw).expect("块前后有字也应抠到");
        assert_eq!(report.did, "d");
        assert_eq!(report.status, "done");
    }

    #[test]
    fn goals_keep_objective_first_then_report_format_then_contract() {
        let goals = build_goals_with_contract(
            "主要目标",
            &["报格式:做了啥".to_string(), "报格式:产出在哪".to_string()],
        );
        assert_eq!(goals[0], "主要目标", "objective 必须仍在首位");
        assert!(goals.iter().any(|g| g == "报格式:做了啥"));
        assert!(goals.iter().any(|g| g == "报格式:产出在哪"));
        assert_eq!(
            goals.last().unwrap(),
            WORKER_REPORT_CONTRACT_TEXT,
            "契约段在最后"
        );
        // done / blocked 的完整样例均显式列出顶层求助字段。
        for key in [
            "完成 done 的完整示例",
            "受阻 blocked 的完整示例",
            "json",
            "did",
            "outputs",
            "status",
            "evidence",
            "permission_requests",
            "open_issues",
            "direction_risks",
            "follow_up_suggestions",
        ] {
            assert!(
                WORKER_REPORT_CONTRACT_TEXT.contains(key),
                "契约段应含 {key}"
            );
        }
        assert!(
            WORKER_REPORT_CONTRACT_TEXT.contains("\"status\": \"done\""),
            "done 示例必须是完整 JSON"
        );
        assert!(
            WORKER_REPORT_CONTRACT_TEXT.contains("\"status\": \"blocked\""),
            "blocked 示例必须是完整 JSON"
        );
    }

    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_dir(tag: &str) -> PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("worker-report-{tag}-{uniq}"));
        fs::create_dir_all(&dir).expect("tmp dir");
        dir
    }

    /// 手写满足登记机器校验的最小 store（workflow/work_item/node + schema）——自包含、不依赖 lib.rs helper。
    fn write_fixture_store(dir: &Path) -> PathBuf {
        let store = serde_json::json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": "seed",
            "projects": [],
            "agent_adapters": [],
            "workflows": [{"workflow_id": "wf-1"}],
            "nodes": [{"workflow_id": "wf-1", "node_id": "wf-1:node:director"}],
            "edges": [],
            "work_items": [{"workflow_id": "wf-1", "work_item_id": "wi-1", "state": "ready_for_review"}],
            "artifacts": [],
            "reviews": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": []
        });
        let path = dir.join("workflow-state.v0.json");
        fs::write(&path, serde_json::to_string_pretty(&store).unwrap()).expect("write store");
        path
    }

    const GOOD_MSG: &str = "干完了。\n```json\n{\"did\":\"改了登录\",\"outputs\":[\"/p/login.tsx\"],\"status\":\"done\",\"evidence\":[\"cargo test 绿\"]}\n```";

    #[test]
    fn consume_good_block_records_to_store() {
        let dir = tmp_dir("good");
        let path = write_fixture_store(&dir);
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            None,
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(outcome.report_summary.is_some(), "解析成功应有摘要");
        assert!(outcome.report_summary.as_deref().unwrap().contains("done"));
        assert!(
            outcome.report_warning.is_none(),
            "落库成功不该有诊断 warning：{:?}",
            outcome.report_warning
        );
        // 断言登记机器**真跑过**：store 里有 worker_structured_report_recorded 审计。
        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let recorded = after["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_type"] == "worker_structured_report_recorded");
        assert!(recorded, "store 应有 worker 报文审计（经登记机器校验落库）");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn consume_no_block_soft_lands_without_write() {
        let dir = tmp_dir("noblock");
        let path = write_fixture_store(&dir);
        let before = fs::read(&path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            None,
            "developer",
            "任务T",
            "我做完了但忘了给 json 块",
        );
        assert!(outcome.report_summary.is_none(), "无块 → 无摘要");
        assert!(
            outcome
                .report_warning
                .as_deref()
                .unwrap_or("")
                .contains("未按契约"),
            "有输出无契约块 → step 级诊断 warning：{:?}",
            outcome.report_warning
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            before,
            "无块不写 store（软着陆·任务仍算完成）"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn consume_record_failure_warns_without_breaking() {
        let dir = tmp_dir("fail");
        let path = write_fixture_store(&dir);
        // work_item_id 不存在 → 登记机器校验 Err → best-effort warning、不断链、不 panic。
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-DOES-NOT-EXIST",
            None,
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(outcome.report_summary.is_some(), "解析成功仍给摘要");
        assert!(
            outcome
                .report_warning
                .as_deref()
                .unwrap_or("")
                .contains("落库失败"),
            "应出落库失败 step 级诊断 warning：{:?}",
            outcome.report_warning
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn consume_blocked_report_returns_help_signal_and_records_real_fields() {
        let dir = tmp_dir("blocked");
        let path = write_fixture_store(&dir);
        let msg = "我需要主管处理。\n```json\n{\"did\":\"权限不足，无法继续\",\"outputs\":[],\"status\":\"blocked\",\"evidence\":[\"读 /secure 被拒\"],\"permission_requests\":[\"请授权读取 /secure\"],\"open_issues\":[\"缺少真实配置文件\"],\"direction_risks\":[\"继续猜会误改沙箱\"],\"follow_up_suggestions\":[\"主管补充路径后重派\"]}\n```";
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            None,
            "developer",
            "任务T",
            msg,
        );
        let help = outcome.help_signal.as_ref().expect("blocked 应返求助信号");
        assert_eq!(help.status, "blocked");
        assert!(help.summary.contains("权限不足"));
        assert_eq!(help.permission_requests, vec!["请授权读取 /secure"]);
        assert_eq!(help.open_issues, vec!["缺少真实配置文件"]);
        assert_eq!(help.direction_risks, vec!["继续猜会误改沙箱"]);
        assert_eq!(help.follow_up_suggestions, vec!["主管补充路径后重派"]);
        assert_eq!(outcome.report_summary, None, "求助不是完成摘要");
        assert_eq!(outcome.report_status.as_deref(), Some("blocked"));

        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let recorded = after["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| event["event_type"] == "worker_structured_report_recorded")
            .expect("求助也应经登记机器落库，成为唯一真源");
        assert_eq!(recorded["acceptance_status"], "blocked");
        assert_eq!(recorded["permission_requests"][0], "请授权读取 /secure");
        assert_eq!(recorded["open_issues"][0], "缺少真实配置文件");
        assert_eq!(recorded["direction_risks"][0], "继续猜会误改沙箱");
        assert_eq!(recorded["follow_up_suggestions"][0], "主管补充路径后重派");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn consume_suspected_help_without_valid_json_escalates_to_help_signal() {
        let dir = tmp_dir("suspected-help");
        let path = write_fixture_store(&dir);
        let before = fs::read(&path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            None,
            "developer",
            "任务T",
            "我卡住了，需要权限读取 /secure。\n```json\n{\"status\":\"blocked\",\n```",
        );
        let help = outcome.help_signal.expect("疑似求助坏 json 应升级");
        assert_eq!(help.status, "suspected_blocked");
        assert!(help.summary.contains("疑似求助"));
        assert!(help.open_issues.iter().any(|item| item.contains("我卡住了")));
        assert!(outcome.report_warning.is_none(), "疑似求助不能降成普通 warning");
        assert_eq!(fs::read(&path).unwrap(), before, "坏 json 不写 store");
        let _ = fs::remove_dir_all(dir);
    }

    // 刀A·口供上脸：consume 把 worker 自报 status 透传进 report_status（done/partial/缺失三态）。
    #[test]
    fn report_status_passthrough_three_states() {
        let dir = tmp_dir("status");
        let path = write_fixture_store(&dir);
        let run = |msg: &str| {
            consume_worker_report_after_completion(
                &path,
                "/p",
                "proj",
                "wf-1",
                "wf-1:node:director",
                "wi-1",
                None,
                "developer",
                "任务T",
                msg,
            )
        };
        assert_eq!(
            run("```json\n{\"did\":\"d\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"e\"]}\n```")
                .report_status
                .as_deref(),
            Some("done"),
            "done 透传"
        );
        assert_eq!(
            run("```json\n{\"did\":\"d\",\"outputs\":[],\"status\":\"partial\",\"evidence\":[\"e\"]}\n```")
                .report_status
                .as_deref(),
            Some("partial"),
            "partial 透传"
        );
        assert_eq!(
            run("```json\n{\"did\":\"d\",\"outputs\":[],\"evidence\":[\"e\"]}\n```").report_status,
            None,
            "有块缺 status → None"
        );
        assert_eq!(run("没有 json 块").report_status, None, "没交口供 → None");
        let _ = fs::remove_dir_all(dir);
    }
}
