// worker 回程契约：报文从「嘱咐」变「契约」。
//
// 任务包：tasks/2026-07-06-worker-report-contract-backend-v1.md
//
// 本模块只定义「契约 + 解析 + 链消费的可测核心」：
//   - 契约文本（确定性追加给 worker，不经 LM）；
//   - 从 worker 最后消息抠出 json 块并解析（软着陆：抠不到/坏 json → None，不 Err）；
//   - 链每任务完成后消费一次：解析 → 组登记入参 → best-effort 调现成登记机器落库 → 出摘要。
//
// 安全属性（安全死线）：**只归档不驱动**——本模块不改任何执行决策/成败/重试/状态迁移；
// 落库走现成 `record_worker_structured_report_at`（自带校验），best-effort（失败只出 warning、不断链）。

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
}

/// 追加给 worker 的契约段（确定性文本·不经 LM·同 consultant/director 的 json 块成熟套路）。
pub(crate) const WORKER_REPORT_CONTRACT_TEXT: &str = "回程契约（务必遵守）：干完后，最后输出**且仅输出**一个 ```json 代码块，严格形如 {\"did\":\"一句话说清做了什么\",\"outputs\":[\"产出文件的完整路径\"],\"status\":\"done|partial|failed\",\"evidence\":[\"怎么证明：命令输出/文件/测试名\"]}。outputs 写产出文件的完整路径；没有产出就写空数组 []。不要在这个 json 块之后再写任何字。";

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
/// 自带校验）→ 出摘要。**只归档不驱动**：无论解析/落库成败都不改任务成败（调用方 completed 仍 completed）。
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
                    report_summary: Some(summary),
                    report_warning: None,
                    report_status: report_status_field(&report.status),
                },
                Err(err) => WorkerReportConsumeOutcome {
                    report_summary: Some(summary),
                    report_warning: Some(format!(
                        "任务「{task_title}」报文落库失败（不影响任务完成）：{err}"
                    )),
                    report_status: report_status_field(&report.status),
                },
            }
        }
        None => {
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
            }
        }
    }
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
    // 契约 status（done|partial|failed）→ 登记机器 acceptance_status 白名单
    // （reported_completed|reported_not_completed|blocked|needs_rework）。
    // 空/未知保守映射为 reported_not_completed（不谎报完成）。
    let acceptance_status = match status.to_lowercase().as_str() {
        "done" => "reported_completed",
        "partial" => "needs_rework",
        "failed" => "reported_not_completed",
        _ => "reported_not_completed",
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
        open_issues: Vec::new(),
        permission_requests: Vec::new(),
        direction_risks: Vec::new(),
        follow_up_suggestions: Vec::new(),
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
        // 契约段含关键约束字段。
        for key in ["json", "did", "outputs", "status", "evidence"] {
            assert!(
                WORKER_REPORT_CONTRACT_TEXT.contains(key),
                "契约段应含 {key}"
            );
        }
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
