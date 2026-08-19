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

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 只读复核 worker 为字节级验收交回的机器可核实证。
///
/// 保持为独立数组字段：旧的 `evidence` 仍承载自然语言/命令摘要，不能被终标机械闸当作
/// 字节、换行或哈希事实。字段缺失保持软着陆；是否足以终标由 supervisor 的授权 check
/// 覆盖闸判定。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct WorkerReviewEvidence {
    #[serde(default)]
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) byte_count: Option<u64>,
    #[serde(default)]
    pub(crate) sha256: String,
    #[serde(default)]
    pub(crate) trailing_newline: Option<bool>,
    #[serde(default)]
    pub(crate) read_method: String,
}

/// worker 回程契约结构：做了啥 / 产出在哪（路径列表）/ 成败 / 怎么证明 / 结论条目。
/// 全 `#[serde(default)]`——缺字段不报错，配合软着陆语义。
///
/// SYN-FND-004B: 新增 report_kind 字段区分执行型/手动型/离线型报告。
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
    /// 只读复核 worker 的结构化文件实证。固定为数组（即使只核一个文件也用一项数组），
    /// 不接受自然语言代替；由 supervisor 根据授权的字节/大小/换行/哈希 checks 机械核覆盖。
    #[serde(default)]
    pub(crate) review_evidence: Vec<WorkerReviewEvidence>,
    /// 只读/分析/审查/盘点类单的结论正文：每条一行，带 file:line + 原文引用。
    /// 写单（改代码/文件）用 outputs/evidence 即可、findings 留空——它不是求助字段，
    /// 不触发 blocked。加它是因为原 did/outputs/evidence 契约为写单设计、装不下只读单的
    /// 报告正文（2026-07-12 站 3b 首单实证：worker 侦察清单被结构化回程丢弃、主管误判证据不足）。
    #[serde(default)]
    pub(crate) findings: Vec<String>,
    #[serde(default)]
    pub(crate) permission_requests: Vec<String>,
    #[serde(default)]
    pub(crate) open_issues: Vec<String>,
    #[serde(default)]
    pub(crate) direction_risks: Vec<String>,
    #[serde(default)]
    pub(crate) follow_up_suggestions: Vec<String>,
    /// SYN-FND-004B: 报告类型。
    /// "execution" = 真实 Codex 执行后的回程报告
    /// "manual" = 手动粘贴的离线报告（不冒充真实执行）
    /// "offline" = 完全离线的手动输入
    ///
    /// **本字段是 worker 自报的**（从 worker 交回的 json 块反序列化，缺省
    /// "execution" 以兼容旧报文），因此**不可作为「这份报告真的来自一次执行」
    /// 的凭据**。`consume_worker_report_after_completion` 会在服务端把它覆盖成
    /// "execution"；除该入口外，任何读到此字段的地方都必须假设它可被伪造。
    #[serde(default = "default_report_kind")]
    pub(crate) report_kind: String,
}

/// SYN-FND-004B: 真实执行回程报告的 report_kind 取值。
pub(crate) const EXECUTION_REPORT_KIND: &str = "execution";

fn default_report_kind() -> String {
    EXECUTION_REPORT_KIND.to_string()
}

/// SYN-FND-004B: 服务端覆盖 worker 自报的 report_kind。
///
/// **作用边界（别高估）**：`report_kind` 当前**没有进 store**——
/// `WorkerStructuredReportInput` 里没有这个字段，`record_worker_structured_report_at`
/// 写的审计事件也不含它。它唯一的下游读者是 `build_report_input` 里的报文哈希预像。
/// 所以本函数保证的是「哈希预像里的 kind 不受 worker 摆布」，
/// **不是**「store 里记下了这份报告的类型」。
fn stamp_execution_report_kind(report: &mut WorkerReport) {
    report.report_kind = EXECUTION_REPORT_KIND.to_string();
}

/// SYN-FND-004B: 执行型报告允许的 attempt 状态白名单。
/// 不在白名单中的状态不允许出现在执行型报告中。
pub(crate) const EXECUTION_REPORT_ALLOWED_ATTEMPT_STATES: &[&str] = &[
    "completed",
    "completed_with_warnings",
    "failed",
    "timed_out",
    "cancelled",
    "blocked",
];

/// SYN-FND-004B: 验证执行型报告的 attempt 状态是否合法。
/// 手动/离线报告不受此约束。
///
/// **已接线**：`consume_worker_report_after_completion` 在落库前调用本函数，
/// 该路径 report_kind 恒为 "execution"，故执行回程恒受白名单约束。
/// 生产调用点只有 director_agent 的两处 completed 分支（该处状态恒为
/// "completed"）；白名单的真正价值在挡住**未来**以中间态（running 等）
/// 回程落库的路径。
pub(crate) fn validate_execution_report_attempt_state(
    report_kind: &str,
    attempt_state: &str,
) -> Result<(), String> {
    if report_kind != "execution" {
        return Ok(());
    }
    if EXECUTION_REPORT_ALLOWED_ATTEMPT_STATES.contains(&attempt_state) {
        Ok(())
    } else {
        Err(format!(
            "fnd004b_rejected: 执行型报告不允许 attempt 状态 '{attempt_state}'，允许: {:?}",
            EXECUTION_REPORT_ALLOWED_ATTEMPT_STATES
        ))
    }
}

/// 追加给 worker 的契约段（确定性文本·不经 LM·同 consultant/director 的 json 块成熟套路）。
pub(crate) const WORKER_REPORT_CONTRACT_TEXT: &str = r#"回程契约（务必遵守）：干完后，最后输出**且仅输出**一个 ```json 代码块。`did`、`outputs`、`status`、`evidence`、`review_evidence`、`findings` 和全部求助字段都只能位于 JSON 顶层；不得嵌套在 `target` 或其他对象中。outputs 写产出文件的完整路径；没有产出就写空数组 []。完成路只使用 done|partial|failed；被阻塞、需要更多权限或资料、或认为方向可能错时，status 必须为 blocked。

**findings（结论正文）**：只读/分析/审查/盘点类任务的结论主体放这里——每条一行字符串，带准确的 file:line + 原文引用（例：`"game.js:137 移动按帧执行未按 delta 缩放，原文 \"player.x += player.vx;\""`）。逐条判定、问题清单、总评都作为 findings 的条目。改代码/写文件类任务用 outputs/evidence 即可、findings 留空。findings 不是求助字段，填了不会被判为受阻。**不要自造 promise_verdicts、top_5_issues 等顶层字段——它们会被丢弃；结论一律进 findings。**

**review_evidence（仅主管明确派发的只读复核单填写）**：这是数组，不是自然语言。每个被核文件交一项 `{"path":"绝对路径","byte_count":8,"sha256":"64 位十六进制 SHA-256","trailing_newline":false,"read_method":"实际使用的只读核验方法"}`。即使只核一个文件也必须写数组；没有该复核要求时写空数组 `[]`。不要把它塞进 evidence 字符串，也不要拿执行 worker 的口供替代只读复核实证。

完成 done 的完整示例（改文件类）：
```json
{
  "did": "创建目标文件并完成回读和字节验证",
  "outputs": ["/绝对路径/目标文件.txt"],
  "status": "done",
  "evidence": ["回读输出与字节校验命令结果"],
  "review_evidence": [],
  "findings": [],
  "permission_requests": [],
  "open_issues": [],
  "direction_risks": [],
  "follow_up_suggestions": []
}
```

完成 done 的完整示例（只读/盘点类）：
```json
{
  "did": "对照 README 逐条核验并按影响排序给出前 5 项问题；未写入任何文件",
  "outputs": [],
  "status": "done",
  "evidence": ["`node --check game.js` 退出码 0"],
  "review_evidence": [],
  "findings": [
    "承诺『A/D 左右移动』已实现，README.md:11 原文 \"`A`/`D` 或方向键左右：移动\"，源码 game.js:119 原文 \"const left = keys.has(\\\"ArrowLeft\\\") ...\"",
    "P0 game.js:137 移动按帧执行未按 delta 缩放，原文 \"player.x += player.vx;\"，高刷会显著加快游戏",
    "总评：核心玩法齐全，主要风险在帧率相关手感与碰撞状态机"
  ],
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
  "review_evidence": [],
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
    /// M2 has no persisted ExecutedReport claim ledger yet. A valid
    /// grant-bearing report therefore stops at an explicit typed boundary;
    /// it neither creates a pseudo receipt nor writes any projection state.
    pub(crate) grant_bearing_boundary: Option<GrantBearingReportBoundary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GrantBearingReportBoundary {
    NotMigratedHold,
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

/// Reload the grant-bearing dispatch from the same authority used by the
/// execution path.  A DB-primary process may deliberately have a stale JSON
/// projection after a committed write, so report admission must never use the
/// projection as its authorization source.
fn load_persisted_dispatch_for_grant_verification(
    state_path: &Path,
    dispatch_id: &str,
) -> Result<Value, String> {
    match crate::workbench_sqlite_storage_mode::storage_mode_for(state_path) {
        crate::workbench_sqlite_storage_mode::StorageMode::DbPrimaryJsonProjection(config) => {
            let connection =
                Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .map_err(|_| "execution_grant_db_primary_dispatch_open_failed".to_string())?;
            let record_json = connection
                .query_row(
                    "SELECT record_json FROM workflow_node_dispatches WHERE dispatch_id = ?1",
                    [dispatch_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|_| "execution_grant_db_primary_dispatch_query_failed".to_string())?
                .ok_or_else(|| "execution_grant_dispatch_not_found".to_string())?;
            serde_json::from_str(&record_json)
                .map_err(|_| "execution_grant_db_primary_dispatch_record_invalid".to_string())
        }
        crate::workbench_sqlite_storage_mode::StorageMode::JsonOnly { .. } => {
            let value = crate::read_workflow_state_value(state_path)?;
            value
                .get("workflow_node_dispatches")
                .and_then(Value::as_array)
                .and_then(|dispatches| {
                    dispatches.iter().find(|candidate| {
                        crate::optional_string_from(candidate, "dispatch_id").as_deref()
                            == Some(dispatch_id)
                    })
                })
                .cloned()
                .ok_or_else(|| "execution_grant_dispatch_not_found".to_string())
        }
    }
}

/// The report path rechecks the *current* exact work-item binding instead of
/// trusting the binding copied into a dispatch at runner start.  This closes a
/// rebind/revocation window between dispatch and report finalization.
fn load_persisted_binding_for_grant_verification(
    state_path: &Path,
    binding_id: &str,
) -> Result<Value, String> {
    match crate::workbench_sqlite_storage_mode::storage_mode_for(state_path) {
        crate::workbench_sqlite_storage_mode::StorageMode::DbPrimaryJsonProjection(config) => {
            let connection =
                Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .map_err(|_| "execution_grant_db_primary_binding_open_failed".to_string())?;
            let record_json = connection
                .query_row(
                    "SELECT record_json FROM workflow_node_session_bindings WHERE binding_id = ?1",
                    [binding_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|_| "execution_grant_db_primary_binding_query_failed".to_string())?
                .ok_or_else(|| "execution_grant_exact_work_item_binding_missing".to_string())?;
            serde_json::from_str(&record_json)
                .map_err(|_| "execution_grant_db_primary_binding_record_invalid".to_string())
        }
        crate::workbench_sqlite_storage_mode::StorageMode::JsonOnly { .. } => {
            let value = crate::read_workflow_state_value(state_path)?;
            value
                .get("workflow_node_session_bindings")
                .and_then(Value::as_array)
                .and_then(|bindings| {
                    bindings.iter().find(|candidate| {
                        crate::optional_string_from(candidate, "binding_id").as_deref()
                            == Some(binding_id)
                    })
                })
                .cloned()
                .ok_or_else(|| "execution_grant_exact_work_item_binding_missing".to_string())
        }
    }
}

fn verify_current_persisted_dispatch_binding_for_report(
    state_path: &Path,
    binding_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    authenticated_actor_id: &str,
) -> Result<(), String> {
    let binding = load_persisted_binding_for_grant_verification(state_path, binding_id)?;
    for (field, expected) in [
        ("binding_id", binding_id),
        ("workflow_id", workflow_id),
        ("node_id", workflow_node_id),
        ("work_item_id", work_item_id),
        ("native_thread_id", authenticated_actor_id),
        ("lifecycle", "active"),
    ] {
        if crate::optional_string_from(&binding, field).as_deref() != Some(expected) {
            return Err(format!(
                "execution_grant_exact_work_item_binding_{field}_mismatch"
            ));
        }
    }
    Ok(())
}

fn verify_persisted_dispatch_grant_for_report(
    state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    attempt_id: Option<&str>,
    attempt_state: &str,
    authenticated_actor_id: &str,
    grant_id: Option<&str>,
    actor_role: &str,
) -> Result<(), String> {
    // Reject a missing or malformed authorization reference before reading a
    // caller-selected state path. The remaining IDs are checked against the
    // reloaded canonical dispatch record immediately afterwards.
    let grant_id = grant_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "execution_grant_id_missing".to_string())?;
    if !is_canonical_execution_grant_id(grant_id) {
        return Err("execution_grant_id_invalid".to_string());
    }
    let dispatch_id = dispatch_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "execution_grant_dispatch_id_missing".to_string())?;
    let attempt_id = attempt_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "execution_grant_attempt_id_missing".to_string())?;
    let dispatch = load_persisted_dispatch_for_grant_verification(state_path, dispatch_id)?;
    for (field, expected) in [
        ("project_id", project_id),
        ("workflow_id", workflow_id),
        ("node_id", workflow_node_id),
        ("work_item_id", work_item_id),
        ("native_thread_id", authenticated_actor_id),
        ("state", attempt_state),
        ("execution_grant_id", grant_id),
        ("execution_attempt_id", attempt_id),
    ] {
        let actual = crate::optional_string_from(&dispatch, field)
            .unwrap_or_else(|| "<missing>".to_string());
        if actual != expected {
            return Err(format!("execution_grant_dispatch_{field}_mismatch"));
        }
    }
    let binding_id = crate::optional_string_from(&dispatch, "binding_id")
        .ok_or_else(|| "execution_grant_binding_id_missing".to_string())?;
    verify_current_persisted_dispatch_binding_for_report(
        state_path,
        &binding_id,
        workflow_id,
        workflow_node_id,
        work_item_id,
        authenticated_actor_id,
    )?;
    let grant: crate::mcp::execution_grant::ExecutionGrant = serde_json::from_value(
        dispatch
            .get("execution_grant")
            .cloned()
            .ok_or_else(|| "execution_grant_persisted_record_missing".to_string())?,
    )
    .map_err(|_| "execution_grant_persisted_record_invalid".to_string())?;
    if grant.grant_id.0 != grant_id {
        return Err("execution_grant_persisted_id_mismatch".to_string());
    }
    let source = crate::plan_authorization_store::load_active_execution_grant_source(
        state_path,
        &grant.authorization_id,
        project_id,
        workflow_id,
        worker_report_timestamp_ms(),
    )
    .map_err(|reason| format!("execution_grant_authorization_source_rejected:{reason}"))?;
    crate::mcp::execution_grant::verify_dispatch_grant_authorization_source(&grant, &source)
        .map_err(|reason| format!("execution_grant_authorization_source_rejected:{reason}"))?;
    match crate::mcp::execution_grant::verify_dispatch_grant(
        &grant,
        &crate::mcp::execution_grant::DispatchGrantVerificationContext {
            project_id,
            workflow_id,
            workflow_node_id,
            work_item_id,
            dispatch_id,
            attempt_id,
            binding_id: &binding_id,
            principal: authenticated_actor_id,
            actor_role,
        },
    ) {
        crate::mcp::execution_grant::GrantVerification::Valid => Ok(()),
        result => Err(format!("execution_grant_verification_rejected:{result:?}")),
    }
}

/// M1 dispatches predate the M2 grant envelope.  This narrow compatibility
/// check is only for a persisted dispatch that proves it has no grant at all;
/// it cannot be used to strip a grant from an M2 report because the canonical
/// record is reloaded before any report write.
fn verify_persisted_legacy_dispatch_for_report(
    state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    attempt_state: &str,
    authenticated_actor_id: &str,
) -> Result<(), String> {
    let dispatch_id = dispatch_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "legacy_m1_dispatch_id_missing".to_string())?;
    let dispatch = load_persisted_dispatch_for_grant_verification(state_path, dispatch_id)?;
    for (field, expected) in [
        ("project_id", project_id),
        ("workflow_id", workflow_id),
        ("node_id", workflow_node_id),
        ("work_item_id", work_item_id),
        ("native_thread_id", authenticated_actor_id),
        ("state", attempt_state),
    ] {
        let actual = crate::optional_string_from(&dispatch, field)
            .unwrap_or_else(|| "<missing>".to_string());
        if actual != expected {
            return Err(format!("legacy_m1_dispatch_{field}_mismatch"));
        }
    }
    if crate::optional_string_from(&dispatch, "execution_grant_id")
        .is_some_and(|grant_id| !grant_id.trim().is_empty())
        || dispatch
            .get("execution_grant")
            .is_some_and(|grant| !grant.is_null())
    {
        return Err("legacy_m1_dispatch_carries_m2_grant".to_string());
    }
    Ok(())
}

fn worker_report_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn is_canonical_execution_grant_id(value: &str) -> bool {
    value
        .strip_prefix("grant:")
        .filter(|suffix| suffix.len() == 64)
        .is_some_and(|suffix| suffix.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

/// 链每任务**完成后**消费一次 worker 最后消息（全文）：解析 → best-effort 落库（现成登记机器·
/// 自带校验）→ 出摘要或求助强信号。
/// - Some(report) → 组登记入参调 `record_worker_structured_report_at`；落库失败仅出 warning、不断链；
///   summary = did（status）。
/// - None（无块/坏 json）→ warning（附原文尾 200 字）+ summary None；任务仍算完成（软着陆）。
///
/// SYN-FND-004B: 新增 attempt_id、authenticated_actor 参数，精确绑定执行上下文。
/// SYN-M2A-T4: grant_id 只是对 server-owned dispatch ledger 的引用；本入口会重读
/// 持久化 dispatch/grant，绝不根据调用方字段自铸或补全授权。
/// SYN-FND-004B: 新增 attempt_state 参数，执行型报告只接受白名单内的 attempt 状态
///（completed/failed/timed_out 等终态；running/dispatched 等中间态拒绝落库）。
/// 本入口进来即无条件把 report_kind 盖成 "execution"——不是靠代码判断来源，
/// 是靠调用路径：只有真实执行回程走到这里。边界见 `stamp_execution_report_kind`。
#[allow(clippy::too_many_arguments)]
pub(crate) fn consume_worker_report_after_completion(
    state_path: &Path,
    project_root: &str,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    attempt_id: Option<&str>,
    // SYN-FND-004B: 该次尝试的真实状态（服务端 dispatch 记录，非 worker 自报）
    attempt_state: &str,
    // SYN-FND-004B: 真实执行者身份（服务端派生，非 project_id）
    authenticated_actor_id: &str,
    // SYN-FND-004C: 执行授权 grant_id（服务端校验过，None = 无授权）
    grant_id: Option<&str>,
    actor_role: &str,
    task_title: &str,
    last_message_full: &str,
) -> WorkerReportConsumeOutcome {
    if let Err(reason) = verify_persisted_dispatch_grant_for_report(
        state_path,
        project_id,
        workflow_id,
        workflow_node_id,
        work_item_id,
        dispatch_id,
        attempt_id,
        attempt_state,
        authenticated_actor_id,
        grant_id,
        actor_role,
    ) {
        return WorkerReportConsumeOutcome {
            report_summary: None,
            report_warning: Some(format!(
                "任务「{task_title}」报文拒绝落库：真实持久 execution grant 验证失败（M2 fail-closed）：{reason}"
            )),
            report_status: None,
            help_signal: None,
            grant_bearing_boundary: None,
        };
    }
    // The M2 grant boundary may not admit a report claim while its dedicated
    // claim owner is NOT_MIGRATED, but it must still reject a non-terminal
    // execution attempt before returning that boundary result.  Otherwise a
    // mid-flight report could be mistaken for a harmless hold response.
    if let Err(reason) = validate_execution_report_attempt_state("execution", attempt_state) {
        return WorkerReportConsumeOutcome {
            report_summary: None,
            report_warning: Some(format!("任务「{task_title}」报文拒绝落库：{reason}")),
            report_status: None,
            help_signal: None,
            grant_bearing_boundary: None,
        };
    }
    let _ = (
        project_root,
        project_id,
        workflow_id,
        workflow_node_id,
        work_item_id,
        dispatch_id,
        attempt_id,
        attempt_state,
        authenticated_actor_id,
        actor_role,
        last_message_full,
    );
    WorkerReportConsumeOutcome {
        report_summary: None,
        report_warning: Some(format!(
            "任务「{task_title}」grant-bearing ExecutedReport 已完成 grant/source/binding/revocation 校验；M2 claim ledger 尚未迁移，返回 NOT_MIGRATED/HOLD，零持久化写入。"
        )),
        report_status: None,
        help_signal: None,
        grant_bearing_boundary: Some(GrantBearingReportBoundary::NotMigratedHold),
    }
}

/// Frozen M1 report admission retained for a canonical dispatch that contains
/// no M2 grant.  M2 report callers must use `consume_worker_report_after_completion`.
/// This compatibility path is a GUARDED_LEGACY_ADAPTER.
#[allow(clippy::too_many_arguments)]
pub(crate) fn consume_legacy_m1_worker_report_after_completion(
    state_path: &Path,
    project_root: &str,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    attempt_id: Option<&str>,
    attempt_state: &str,
    authenticated_actor_id: &str,
    actor_role: &str,
    task_title: &str,
    last_message_full: &str,
) -> WorkerReportConsumeOutcome {
    if let Err(reason) = verify_persisted_legacy_dispatch_for_report(
        state_path,
        project_id,
        workflow_id,
        workflow_node_id,
        work_item_id,
        dispatch_id,
        attempt_state,
        authenticated_actor_id,
    ) {
        return WorkerReportConsumeOutcome {
            report_summary: None,
            report_warning: Some(format!(
                "任务「{task_title}」报文拒绝落库：M1 兼容派发记录不匹配：{reason}"
            )),
            report_status: None,
            help_signal: None,
            grant_bearing_boundary: None,
        };
    }
    consume_authorized_worker_report_after_completion(
        state_path,
        project_root,
        project_id,
        workflow_id,
        workflow_node_id,
        work_item_id,
        dispatch_id,
        attempt_id,
        attempt_state,
        authenticated_actor_id,
        actor_role,
        task_title,
        last_message_full,
    )
}

#[allow(clippy::too_many_arguments)]
fn consume_authorized_worker_report_after_completion(
    state_path: &Path,
    project_root: &str,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    attempt_id: Option<&str>,
    attempt_state: &str,
    authenticated_actor_id: &str,
    actor_role: &str,
    task_title: &str,
    last_message_full: &str,
) -> WorkerReportConsumeOutcome {
    // SYN-FND-003: 使用 identity_kernel 解析执行者身份（服务端派生，非前端传入）
    let identity = crate::mcp::identity_kernel::resolve_identity(
        authenticated_actor_id,
        project_root,
        actor_role,
        "development", // 真实执行 = development 通道
        false,         // caller_boolean = 输入，不影响解析
    );
    // 身份解析失败 → 拒绝落库（fail closed）
    if let crate::mcp::identity_kernel::IdentityResolution::Denied(reason) = &identity {
        return WorkerReportConsumeOutcome {
            report_summary: None,
            report_warning: Some(format!(
                "任务「{task_title}」报文拒绝落库：身份解析失败（FND-003 fail-closed）：{reason}"
            )),
            report_status: None,
            help_signal: None,
            grant_bearing_boundary: None,
        };
    }

    // SYN-FND-004B: 执行型报告只接受白名单内的 attempt 终态（fail closed）。
    // 本路径 report_kind 恒为 "execution"（见下方 stamp），故恒受白名单约束。
    if let Err(reason) = validate_execution_report_attempt_state("execution", attempt_state) {
        return WorkerReportConsumeOutcome {
            report_summary: None,
            report_warning: Some(format!("任务「{task_title}」报文拒绝落库：{reason}")),
            report_status: None,
            help_signal: None,
            grant_bearing_boundary: None,
        };
    }

    match parse_worker_report(last_message_full) {
        Some(mut report) => {
            // SYN-FND-004B: report_kind 由服务端在此覆盖，不采用 worker 自报值。
            report.report_kind = "execution".to_string();
            // 覆盖能保证什么、不能保证什么，见 `stamp_execution_report_kind` 注释。
            stamp_execution_report_kind(&mut report);

            let summary = worker_report_summary(&report);
            let help_signal = worker_report_help_signal(&report, &summary);
            let input = build_report_input(
                project_root,
                project_id,
                workflow_id,
                workflow_node_id,
                work_item_id,
                dispatch_id,
                attempt_id,
                authenticated_actor_id,
                actor_role,
                &report,
            );
            let record_result = crate::record_worker_structured_report_at(state_path, &input);
            match record_result {
                Ok(_result) => WorkerReportConsumeOutcome {
                    report_summary: if help_signal.is_some() {
                        None
                    } else {
                        Some(summary.clone())
                    },
                    report_warning: None,
                    report_status: report_status_field(&report.status),
                    help_signal,
                    grant_bearing_boundary: None,
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
                    grant_bearing_boundary: None,
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
                    grant_bearing_boundary: None,
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
                grant_bearing_boundary: None,
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
///
/// SYN-FND-004B: 新增 attempt_id、authenticated_actor、report_hash 精确绑定。
#[allow(clippy::too_many_arguments)]
fn build_report_input(
    project_root: &str,
    project_id: &str,
    workflow_id: &str,
    workflow_node_id: &str,
    work_item_id: &str,
    dispatch_id: Option<&str>,
    attempt_id: Option<&str>,
    // SYN-FND-004B: 真实执行者身份（服务端派生，非 project_id）
    authenticated_actor_id: &str,
    actor_role: &str,
    report: &WorkerReport,
) -> crate::WorkerStructuredReportInput {
    let timestamp = crate::unix_timestamp_string();
    // SYN-FND-005: 在构建输入前对报告内容进行敏感分类和脱敏
    let did = crate::mcp::event_audit_boundary::scrub_content(report.did.trim());
    let status = report.status.trim().to_string();
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

    // SYN-FND-004B: 计算报文哈希（检测报文内容是否与登记时一致，非防篡改）
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(report.did.as_bytes());
    hasher.update(report.status.as_bytes());
    for output in &report.outputs {
        hasher.update(output.as_bytes());
    }
    for evidence in &report.evidence {
        hasher.update(evidence.as_bytes());
    }
    hasher.update(report.report_kind.as_bytes());
    let report_hash = format!("sha256:{:064x}", hasher.finalize());

    crate::WorkerStructuredReportInput {
        project_root: project_root.to_string(),
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_node_id: workflow_node_id.to_string(),
        work_item_id: work_item_id.to_string(),
        dispatch_id: dispatch_id.map(str::to_string),
        attempt_id: attempt_id.map(str::to_string),
        execution_grant_id: None,
        authenticated_actor_id: authenticated_actor_id.to_string(),
        authenticated_project_scope: project_id.to_string(),
        report_hash,
        report_kind: report.report_kind.clone(),
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

    /// SYN-FND-004B: worker 在 json 里自称 report_kind 的值必须被服务端覆盖。
    /// 锁的是 `stamp_execution_report_kind`——真实执行回程入口调它。
    #[test]
    fn worker_self_reported_report_kind_is_overridden_server_side() {
        // worker 自称 "manual"，想让这份真实执行的报告看着像手动粘贴的。
        let mut report = parse_worker_report(
            "```json\n{\"did\":\"d\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"e\"],\"report_kind\":\"manual\"}\n```",
        )
        .expect("报文应解析成功");
        assert_eq!(
            report.report_kind, "manual",
            "前置条件：worker 自报值确实进得来（本字段可被伪造，这正是要覆盖它的原因）"
        );

        stamp_execution_report_kind(&mut report);

        assert_eq!(
            report.report_kind, EXECUTION_REPORT_KIND,
            "服务端必须把 report_kind 覆盖成 execution，不采用 worker 自报值"
        );
    }

    /// SYN-FND-004B: 覆盖后的 report_kind 必须真正影响报文哈希预像。
    /// 若哪天 build_report_input 不再把 kind 喂进哈希，这条会红——
    /// 因为那时覆盖就成了纯注释，report_kind 会彻底没有下游读者。
    #[test]
    fn report_kind_override_changes_report_hash_preimage() {
        let base = "```json\n{\"did\":\"d\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"e\"],\"report_kind\":\"manual\"}\n```";
        let self_reported = parse_worker_report(base).expect("报文应解析成功");
        let mut stamped = self_reported.clone();
        stamp_execution_report_kind(&mut stamped);

        let hash_of = |report: &WorkerReport| {
            build_report_input(
                "/p",
                "proj",
                "wf-1",
                "wf-1:node:director",
                "wi-1",
                None,
                None,
                "test-actor",
                "developer",
                report,
            )
            .report_hash
        };

        assert_ne!(
            hash_of(&self_reported),
            hash_of(&stamped),
            "kind 变了哈希就该变；相等说明 report_kind 已不在哈希预像里、覆盖已无下游读者"
        );
    }

    #[test]
    fn parses_full_block() {
        let raw = "干完了。\n```json\n{\"did\":\"改了登录页\",\"outputs\":[\"/p/login.tsx\"],\"status\":\"done\",\"evidence\":[\"cargo test 绿\"]}\n```";
        let report = parse_worker_report(raw).expect("合法块应解析");
        assert_eq!(report.did, "改了登录页");
        assert_eq!(report.outputs, vec!["/p/login.tsx"]);
        assert_eq!(report.status, "done");
        assert_eq!(report.evidence, vec!["cargo test 绿"]);
        assert!(report.review_evidence.is_empty());
    }

    #[test]
    fn parses_machine_review_evidence_array() {
        let raw = "```json\n{\"did\":\"只读复核完成\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"wc 与 sha256\"],\"review_evidence\":[{\"path\":\"/p/output.txt\",\"byte_count\":9,\"sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"trailing_newline\":true,\"read_method\":\"wc -c + sha256sum + tail\"}]}\n```";
        let report = parse_worker_report(raw).expect("数组形态的复核实证应解析");
        assert_eq!(report.review_evidence.len(), 1);
        let evidence = &report.review_evidence[0];
        assert_eq!(evidence.path, "/p/output.txt");
        assert_eq!(evidence.byte_count, Some(9));
        assert_eq!(
            evidence.sha256,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(evidence.trailing_newline, Some(true));
        assert_eq!(evidence.read_method, "wc -c + sha256sum + tail");
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
        assert!(report.review_evidence.is_empty());
        // findings 缺省也是空——写单不填不受影响。
        assert!(report.findings.is_empty());
    }

    #[test]
    fn parses_findings_for_readonly_report() {
        let raw = "```json\n{\"did\":\"只读盘点\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"node --check 退出码 0\"],\"findings\":[\"P0 game.js:137 未按 delta 缩放，原文 \\\"player.x += player.vx;\\\"\",\"总评：核心玩法齐全\"]}\n```";
        let report = parse_worker_report(raw).expect("带 findings 的只读报文应解析");
        assert_eq!(report.status, "done");
        assert_eq!(report.findings.len(), 2);
        assert!(report.findings[0].contains("game.js:137"));
        // findings 不是求助字段：填了不该让求助字段非空。
        assert!(report.permission_requests.is_empty());
        assert!(report.open_issues.is_empty());
    }

    #[test]
    fn station3b_regression_self_invented_fields_dropped_findings_kept() {
        // 站 3b 首单病根：worker 把结论塞进自造的 promise_verdicts/top_5_issues 顶层字段，
        // serde 静默丢弃 → 主管只见摘要、误判证据不足。修法=结论进 findings，未知字段仍丢弃。
        let raw = "```json\n{\"did\":\"盘点完成\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"只读检查\"],\"findings\":[\"README.md:11 承诺移动已实现，源码 game.js:119\"],\"promise_verdicts\":[{\"verdict\":\"已实现\"}],\"top_5_issues\":[{\"rank\":1}]}\n```";
        let report = parse_worker_report(raw).expect("含未知顶层字段仍应解析");
        // 自造字段被丢（struct 里没有它们，serde 忽略），但结论正文经 findings 保住。
        assert_eq!(
            report.findings,
            vec!["README.md:11 承诺移动已实现，源码 game.js:119"]
        );
        assert_eq!(report.did, "盘点完成");
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
            "review_evidence",
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

    struct FixtureStore {
        path: PathBuf,
        dispatch_id: String,
        attempt_id: String,
        grant_id: String,
        authorization_id: String,
        authorization_revision: i64,
    }

    /// 手写满足登记机器校验的最小 store，并把 server-side persisted
    /// grant 放进 canonical dispatch ledger。它不把调用方参数当授权来源。
    fn write_fixture_store(dir: &Path) -> FixtureStore {
        write_fixture_store_with_dispatch(dir, "wi-1", "completed")
    }

    fn write_fixture_store_with_dispatch(
        dir: &Path,
        dispatch_work_item_id: &str,
        dispatch_state: &str,
    ) -> FixtureStore {
        let mut source = crate::mcp::execution_grant::ExecutionGrantAuthorizationSource {
            authorization_id: "plan-auth:fixture".to_string(),
            authorization_store_revision: 1,
            authorization_source_hash: "fixture-source-hash".to_string(),
            project_id: "proj".to_string(),
            workflow_id: "wf-1".to_string(),
            allowed_work_item_types: vec!["task_package".to_string()],
            allowed_role_ids: vec!["developer".to_string()],
            allowed_agent_ids: vec!["test-actor".to_string()],
            allowed_read_roots: vec!["/p".to_string()],
            allowed_write_roots: vec!["/p".to_string()],
            allowed_tools: vec!["cargo-test".to_string()],
            allowed_checks: vec!["cargo-test".to_string()],
            stop_conditions: vec!["user-rejected".to_string()],
            expires_at_ms: None,
            max_worker_dispatches: Some(1),
            max_runtime_minutes: Some(1),
        };
        let binding = crate::mcp::execution_grant::ExecutionGrantBinding {
            dispatch_id: "dispatch:fixture".to_string(),
            project_id: "proj".to_string(),
            workflow_id: "wf-1".to_string(),
            workflow_node_id: "wf-1:node:director".to_string(),
            work_item_id: dispatch_work_item_id.to_string(),
            binding_id: "binding:fixture".to_string(),
            principal: "test-actor".to_string(),
            prepared_dispatch_id: "prepared:fixture".to_string(),
        };
        let authorization_id = source.authorization_id.clone();
        let authorization_revision = source.authorization_store_revision;
        let authorization_store = serde_json::json!({
            "schema_version": "plan_authorization_store.v1",
            "revision": authorization_revision,
            "authorizations": [{
                "authorization_id": authorization_id,
                "schema_version": "plan_authorization.v1",
                "project_id": "proj",
                "workflow_id": "wf-1",
                "source_proposal_id": "proposal:fixture",
                "title": "worker report fixture authorization",
                "goal_summary": "server-owned grant source fixture",
                "status": "active",
                "scope": {
                    "project_id": "proj",
                    "workflow_id": "wf-1",
                    "allowed_role_ids": source.allowed_role_ids,
                    "allowed_agent_ids": source.allowed_agent_ids,
                    "allowed_read_roots": source.allowed_read_roots,
                    "allowed_write_roots": source.allowed_write_roots,
                    "allowed_tools": source.allowed_tools,
                    "allowed_checks": source.allowed_checks,
                    "allowed_task_package_kinds": source.allowed_work_item_types,
                    "max_worker_dispatches": 1,
                    "max_runtime_minutes": 1,
                    "stop_conditions": [{
                        "condition_id": "user-rejected",
                        "kind": "fixture",
                        "summary": "fixture stop condition",
                        "requires_user_confirmation": false
                    }]
                },
                "user_confirmation": {
                    "confirmed_by": "user",
                    "confirmed_at_ms": 1700000000000_i64,
                    "confirmation_summary": "fixture confirmed"
                },
                "global_boundary_review": {
                    "reviewed_by": "global_director",
                    "reviewed_at_ms": 1700000000000_i64,
                    "status": "approved",
                    "summary": "fixture approved",
                    "source_proposal_id": "proposal:fixture",
                    "checklist": null,
                    "findings": [],
                    "reviewed_scope_fingerprint": "fixture"
                },
                "audit_refs": [],
                "created_at_ms": 1700000000000_i64,
                "updated_at_ms": 1700000000000_i64,
                "expires_at_ms": null
            }],
            "audit_events": [],
            "updated_at_ms": 1700000000000_i64,
            "warnings": []
        });
        let authorization: crate::PlanAuthorization =
            serde_json::from_value(authorization_store["authorizations"][0].clone())
                .expect("fixture authorization source");
        source.authorization_source_hash = crate::utils::hash::sha256_hex(
            &serde_json::to_string(&authorization).expect("serialize fixture authorization source"),
        );
        let grant = crate::mcp::execution_grant::mint_dispatch_grant(&source, &binding, 60)
            .expect("fixture persisted grant");
        let grant_id = grant.grant_id.0.clone();
        let attempt_id = grant.attempt_id.clone().expect("fixture attempt");
        let store = serde_json::json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": "seed",
            "projects": [{"project_id": "proj"}],
            "agent_adapters": [],
            "workflows": [{"workflow_id": "wf-1", "project_id": "proj"}],
            "nodes": [{"workflow_id": "wf-1", "node_id": "wf-1:node:director"}],
            "edges": [],
            "work_items": [{"workflow_id": "wf-1", "work_item_id": "wi-1", "state": "ready_for_review"}],
            "workflow_node_session_bindings": [{
                "binding_id": binding.binding_id.clone(),
                "project_id": "proj",
                "workflow_id": "wf-1",
                "node_id": "wf-1:node:director",
                "work_item_id": dispatch_work_item_id,
                "agent_type": "codex",
                "adapter_id": "codex-local",
                "native_thread_id": "test-actor",
                "binding_source": "fixture",
                "binding_mode": "fixture",
                "lifecycle": "active",
                "created_at_ms": 1700000000000_i64,
                "updated_at_ms": 1700000000000_i64,
                "warnings": []
            }],
            "workflow_node_dispatches": [{
                "dispatch_id": binding.dispatch_id.clone(),
                "project_id": "proj",
                "workflow_id": "wf-1",
                "node_id": "wf-1:node:director",
                "work_item_id": dispatch_work_item_id,
                "binding_id": binding.binding_id.clone(),
                "native_thread_id": "test-actor",
                "state": dispatch_state,
                "execution_grant_id": grant_id.clone(),
                "execution_attempt_id": attempt_id.clone(),
                "execution_grant": serde_json::to_value(&grant).expect("serialize fixture grant")
            }],
            "artifacts": [],
            "reviews": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": []
        });
        let path = dir.join("workflow-state.v0.json");
        fs::write(&path, serde_json::to_string_pretty(&store).unwrap()).expect("write store");
        let authorization_path = crate::plan_authorization_store::sidecar_path(&path)
            .expect("fixture authorization sidecar path");
        fs::write(
            authorization_path,
            serde_json::to_vec_pretty(&authorization_store).expect("serialize auth source fixture"),
        )
        .expect("write authorization source fixture");
        FixtureStore {
            path,
            dispatch_id: binding.dispatch_id,
            attempt_id: grant.attempt_id.expect("fixture attempt"),
            grant_id: grant.grant_id.0,
            authorization_id,
            authorization_revision,
        }
    }

    const GOOD_MSG: &str = "干完了。\n```json\n{\"did\":\"改了登录\",\"outputs\":[\"/p/login.tsx\"],\"status\":\"done\",\"evidence\":[\"cargo test 绿\"]}\n```";

    #[test]
    fn grant_bearing_valid_report_returns_not_migrated_hold_without_write() {
        let dir = tmp_dir("good");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",  // attempt_state (SYN-FND-004B): 合法终态
            "test-actor", // authenticated_actor_id (SYN-FND-004B)
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(
            outcome.report_warning.is_some(),
            "M2 必须明确返回 NOT_MIGRATED/HOLD：{:?}",
            outcome.report_warning
        );
        assert_eq!(
            outcome.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold),
            "验证通过的 grant-bearing report 必须命中 typed NOT_MIGRATED/HOLD"
        );
        assert!(outcome.report_summary.is_none());
        assert!(outcome.help_signal.is_none());
        assert!(outcome
            .report_warning
            .as_deref()
            .is_some_and(|warning| warning.contains("NOT_MIGRATED/HOLD")));
        assert_eq!(fs::read(&path).unwrap(), before, "valid M2 hold must not append pseudo claim/audit, update top-level timestamp, or mutate any owner");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn grant_bearing_non_contract_payload_still_returns_hold_without_write() {
        let dir = tmp_dir("noblock");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(&path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",  // attempt_state (SYN-FND-004B): 合法终态
            "test-actor", // authenticated_actor_id (SYN-FND-004B)
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            "我做完了但忘了给 json 块",
        );
        assert!(outcome.report_summary.is_none());
        assert_eq!(
            outcome.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold)
        );
        assert!(outcome
            .report_warning
            .as_deref()
            .is_some_and(|warning| warning.contains("NOT_MIGRATED/HOLD")));
        assert_eq!(
            fs::read(&path).unwrap(),
            before,
            "M2 boundary does not parse or persist the report payload"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn grant_bearing_boundary_does_not_attempt_legacy_report_record() {
        let dir = tmp_dir("fail");
        let fixture = write_fixture_store_with_dispatch(&dir, "wi-DOES-NOT-EXIST", "completed");
        let path = fixture.path.as_path();
        let before = fs::read(path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-DOES-NOT-EXIST",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",  // attempt_state (SYN-FND-004B): 合法终态
            "test-actor", // authenticated_actor_id (SYN-FND-004B)
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert_eq!(
            outcome.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold)
        );
        assert_eq!(
            fs::read(path).unwrap(),
            before,
            "M2 boundary must not fall through to the legacy report recorder"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn grant_bearing_blocked_payload_returns_hold_without_forwarding_or_write() {
        let dir = tmp_dir("blocked");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(path).unwrap();
        let msg = "我需要主管处理。\n```json\n{\"did\":\"权限不足，无法继续\",\"outputs\":[],\"status\":\"blocked\",\"evidence\":[\"读 /secure 被拒\"],\"permission_requests\":[\"请授权读取 /secure\"],\"open_issues\":[\"缺少真实配置文件\"],\"direction_risks\":[\"继续猜会误改沙箱\"],\"follow_up_suggestions\":[\"主管补充路径后重派\"]}\n```";
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",  // attempt_state (SYN-FND-004B): 合法终态
            "test-actor", // authenticated_actor_id (SYN-FND-004B)
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            msg,
        );
        assert_eq!(
            outcome.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold)
        );
        assert!(outcome.help_signal.is_none());
        assert!(outcome.report_status.is_none());
        assert_eq!(
            fs::read(path).unwrap(),
            before,
            "grant-bearing hold must not forward report content to a legacy side effect"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn grant_bearing_hold_is_repeatable_and_payload_independent_zero_write() {
        let dir = tmp_dir("grant-hold-replay");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(path).unwrap();
        let consume = |message: &str| {
            consume_worker_report_after_completion(
                path,
                "/p",
                "proj",
                "wf-1",
                "wf-1:node:director",
                "wi-1",
                Some(fixture.dispatch_id.as_str()),
                Some(fixture.attempt_id.as_str()),
                "completed",
                "test-actor",
                Some(fixture.grant_id.as_str()),
                "developer",
                "任务T",
                message,
            )
        };

        let first = consume(GOOD_MSG);
        assert_eq!(
            first.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold)
        );
        let replay = consume(GOOD_MSG);
        assert_eq!(
            replay.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold),
            "same valid report remains a typed hold until the real claim ledger exists"
        );
        assert_eq!(
            fs::read(path).expect("read replayed claim"),
            before,
            "M2 hold must not append audit, bump state, or rewrite a foreign owner"
        );

        let divergent = consume(
            "```json\n{\"did\":\"different report\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"fixture\"]}\n```",
        );
        assert_eq!(
            divergent.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold),
            "different valid payload cannot create an ad-hoc claim receipt or overwrite a future claim"
        );
        assert_eq!(
            fs::read(path).expect("read divergent claim"),
            before,
            "different valid payload must leave the entire store byte-for-byte unchanged"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn grant_bearing_malformed_payload_returns_hold_without_write() {
        let dir = tmp_dir("suspected-help");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(&path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",  // attempt_state (SYN-FND-004B): 合法终态
            "test-actor", // authenticated_actor_id (SYN-FND-004B)
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            "我卡住了，需要权限读取 /secure。\n```json\n{\"status\":\"blocked\",\n```",
        );
        assert_eq!(
            outcome.grant_bearing_boundary,
            Some(GrantBearingReportBoundary::NotMigratedHold)
        );
        assert!(outcome.report_summary.is_none());
        assert!(outcome.report_status.is_none());
        assert!(outcome.help_signal.is_none());
        assert!(outcome
            .report_warning
            .as_deref()
            .is_some_and(|warning| warning.contains("NOT_MIGRATED/HOLD")));
        assert_eq!(fs::read(&path).unwrap(), before, "坏 json 不写 store");
        let _ = fs::remove_dir_all(dir);
    }

    // M2 grant-bearing reports stop before parsing or forwarding worker-owned
    // status.  The future claim/review owner, not this boundary, owns those
    // semantics.
    #[test]
    fn grant_bearing_reports_do_not_forward_worker_status_before_claim_migration() {
        let dir = tmp_dir("status");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let run = |msg: &str| {
            consume_worker_report_after_completion(
                &path,
                "/p",
                "proj",
                "wf-1",
                "wf-1:node:director",
                "wi-1",
                Some(fixture.dispatch_id.as_str()),
                Some(fixture.attempt_id.as_str()),
                "completed",  // attempt_state (SYN-FND-004B): 合法终态
                "test-actor", // authenticated_actor_id (SYN-FND-004B)
                Some(fixture.grant_id.as_str()),
                "developer",
                "任务T",
                msg,
            )
        };
        for message in [
            "```json\n{\"did\":\"d\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"e\"]}\n```",
            "```json\n{\"did\":\"d\",\"outputs\":[],\"status\":\"partial\",\"evidence\":[\"e\"]}\n```",
            "```json\n{\"did\":\"d\",\"outputs\":[],\"evidence\":[\"e\"]}\n```",
            "没有 json 块",
        ] {
            let outcome = run(message);
            assert_eq!(
                outcome.grant_bearing_boundary,
                Some(GrantBearingReportBoundary::NotMigratedHold),
                "grant-bearing payload must stop at the M2 boundary"
            );
            assert!(outcome.report_status.is_none());
            assert!(outcome.help_signal.is_none());
        }
        let _ = fs::remove_dir_all(dir);
    }

    // SYN-FND-004C: 无 grant_id 的执行回程必须 fail closed——拒绝落库、诊断 warning、store 逐字节不变。
    #[test]
    fn consume_without_grant_id_is_rejected_and_writes_nothing() {
        let dir = tmp_dir("nogrant");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(&path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",  // attempt_state (SYN-FND-004B): 合法终态
            "test-actor", // authenticated_actor_id (SYN-FND-004B)
            None,         // grant_id (SYN-FND-004C): 无授权
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(outcome.report_summary.is_none(), "无 grant 不得产生摘要");
        assert!(outcome.report_status.is_none(), "无 grant 不得产生状态");
        let warning = outcome.report_warning.expect("无 grant 必须有诊断 warning");
        assert!(
            warning.contains("execution_grant_id_missing"),
            "warning 应指明拒绝原因：{warning}"
        );
        assert_eq!(fs::read(&path).unwrap(), before, "拒绝必须零 store 变化");
        let _ = fs::remove_dir_all(dir);
    }

    // SYN-M2A-T4: malformed caller-supplied grant ID is rejected before any store read.
    #[test]
    fn consume_with_malformed_grant_id_is_rejected_and_writes_nothing() {
        let dir = tmp_dir("badgrant");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(&path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",              // attempt_state (SYN-FND-004B): 合法终态
            "test-actor",             // authenticated_actor_id (SYN-FND-004B)
            Some("forged-by-caller"), // grant_id (SYN-FND-004C): 格式非法
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(outcome.report_summary.is_none(), "非法 grant 不得产生摘要");
        let warning = outcome
            .report_warning
            .expect("非法 grant 必须有诊断 warning");
        assert!(
            warning.contains("execution_grant_id_invalid"),
            "warning 应指明拒绝原因：{warning}"
        );
        assert_eq!(fs::read(&path).unwrap(), before, "拒绝必须零 store 变化");
        let _ = fs::remove_dir_all(dir);
    }

    // A caller can mimic the surface shape of a grant ID, but it cannot replace
    // the immutable ID stored in the server-owned dispatch ledger.
    #[test]
    fn consume_with_forged_canonical_grant_id_is_rejected_and_writes_nothing() {
        let dir = tmp_dir("forged-grant");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(path).unwrap();
        let forged_grant_id = format!("grant:{}", "a".repeat(64));
        let outcome = consume_worker_report_after_completion(
            path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",
            "test-actor",
            Some(&forged_grant_id),
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(outcome.report_summary.is_none());
        assert!(outcome
            .report_warning
            .as_deref()
            .unwrap_or("")
            .contains("execution_grant_dispatch_execution_grant_id_mismatch"));
        assert_eq!(
            fs::read(path).unwrap(),
            before,
            "forged grant must not mutate store"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // A copied real grant is also insufficient when the report's server-side
    // actor context does not match the persisted dispatch subject.
    #[test]
    fn consume_with_forged_report_subject_is_rejected_and_writes_nothing() {
        let dir = tmp_dir("forged-report");
        let fixture = write_fixture_store(&dir);
        let path = fixture.path.as_path();
        let before = fs::read(path).unwrap();
        let outcome = consume_worker_report_after_completion(
            path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",
            "forged-actor",
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(outcome.report_summary.is_none());
        assert!(outcome
            .report_warning
            .as_deref()
            .unwrap_or("")
            .contains("execution_grant_dispatch_native_thread_id_mismatch"));
        assert_eq!(
            fs::read(path).unwrap(),
            before,
            "forged report must not mutate store"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn consume_after_authorization_revocation_is_rejected_and_writes_nothing() {
        let dir = tmp_dir("revoked-authority");
        let fixture = write_fixture_store(&dir);
        crate::plan_authorization_store::revoke_authorization(
            &fixture.path,
            &crate::RevokePlanAuthorizationInput {
                project_root: "/p".to_string(),
                authorization_id: fixture.authorization_id.clone(),
                actor_id: "fixture-user".to_string(),
                actor_role: "user".to_string(),
                reason: "fixture revoke after mint".to_string(),
                expected_store_revision: Some(fixture.authorization_revision),
            },
            worker_report_timestamp_ms(),
            "worker-report-revoke-fixture",
        )
        .expect("revoke server authorization after grant mint");
        let state_before = fs::read(&fixture.path).expect("read state after revoke");
        let authorization_path = crate::plan_authorization_store::sidecar_path(&fixture.path)
            .expect("authorization sidecar path");
        let authorization_before =
            fs::read(&authorization_path).expect("read sidecar after revoke");

        let outcome = consume_worker_report_after_completion(
            &fixture.path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "completed",
            "test-actor",
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(
            outcome.report_summary.is_none(),
            "revoked source must not produce a report"
        );
        assert!(
            outcome.report_status.is_none(),
            "revoked source must not produce a status"
        );
        assert!(
            outcome
                .report_warning
                .as_deref()
                .unwrap_or("")
                .contains("execution_grant_authorization_source_rejected:execution_grant_authorization_not_active"),
            "revocation must reject the previously minted grant: {:?}",
            outcome.report_warning
        );
        assert_eq!(
            fs::read(&fixture.path).expect("read state after rejected report"),
            state_before,
            "rejected post-revocation report must not write workflow state"
        );
        assert_eq!(
            fs::read(&authorization_path).expect("read sidecar after rejected report"),
            authorization_before,
            "rejected post-revocation report must not write authorization source"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // SYN-FND-004B: 中间态（非白名单）的 attempt 回程必须 fail closed——合法 grant 也救不回。
    #[test]
    fn consume_with_mid_flight_attempt_state_is_rejected_and_writes_nothing() {
        let dir = tmp_dir("midflight");
        let fixture = write_fixture_store_with_dispatch(&dir, "wi-1", "running");
        let path = fixture.path.as_path();
        let before = fs::read(&path).unwrap();
        let outcome = consume_worker_report_after_completion(
            &path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            Some(fixture.dispatch_id.as_str()),
            Some(fixture.attempt_id.as_str()),
            "running",    // attempt_state (SYN-FND-004B): 中间态，不在白名单
            "test-actor", // authenticated_actor_id (SYN-FND-004B)
            Some(fixture.grant_id.as_str()),
            "developer",
            "任务T",
            GOOD_MSG,
        );
        assert!(outcome.report_summary.is_none(), "中间态不得产生摘要");
        let warning = outcome.report_warning.expect("中间态必须有诊断 warning");
        assert!(
            warning.contains("fnd004b_rejected"),
            "warning 应含白名单拒绝码：{warning}"
        );
        assert_eq!(fs::read(&path).unwrap(), before, "拒绝必须零 store 变化");
        let _ = fs::remove_dir_all(dir);
    }
}

// SYN-PRJ-001 / M5R01: M5 WorkerReport 分型与精确执行 join
//
// 本合同（docs/contracts/m5-execution-identity-and-worker-report-v1.md）冻结：
// executed report 必须完整核对 ProjectId + CorrelationId/OrchestrationId +
// WorkflowRunId + WorkItemId + NodeId + DispatchId + AttemptId + GrantId +
// worker RoleSessionId + authoritative receipt + trusted actor + hash；
// executed/manual/offline 彻底分型；缺省 ReportKind 不得自动成为执行报告。

/// 报告类型枚举
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) enum ReportKind {
    /// 真实执行后的回程报告
    Execution,
    /// 手动粘贴的离线报告
    Manual,
    /// 完全离线的手动输入
    Offline,
}

impl Default for ReportKind {
    fn default() -> Self {
        // M5R01: 缺省必须是不可执行的 Manual；禁止缺省 ReportKind 自动成为 Execution
        Self::Manual
    }
}

impl std::fmt::Display for ReportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportKind::Execution => write!(f, "executed"),
            ReportKind::Manual => write!(f, "manual"),
            ReportKind::Offline => write!(f, "offline"),
        }
    }
}

impl ReportKind {
    /// 从字符串解析（合同规定值：executed / manual / offline）
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "executed" | "execution" => Some(Self::Execution),
            "manual" => Some(Self::Manual),
            "offline" => Some(Self::Offline),
            _ => None,
        }
    }

    /// 是否为真实执行报告
    pub(crate) fn is_execution(&self) -> bool {
        matches!(self, Self::Execution)
    }

    /// 是否为手动/离线报告（不冒充真实执行）
    pub(crate) fn is_manual_or_offline(&self) -> bool {
        matches!(self, Self::Manual | Self::Offline)
    }
}

/// 执行回执 - 真实执行的机器可核实证
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ExecutionReceipt {
    /// 执行 ID
    pub execution_id: String,
    /// 执行开始时间戳 (ms)
    pub started_at_ms: i64,
    /// 执行结束时间戳 (ms)
    pub completed_at_ms: Option<i64>,
    /// 执行状态
    pub status: String,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 输出哈希
    pub output_hash: Option<String>,
    /// 成本 (tokens)
    pub cost_tokens: Option<u64>,
}

/// 可信执行者 - actor 只来自可信 session/binding，不来自 report 自报
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct TrustedActor {
    /// 执行者 ID
    pub actor_id: String,
    /// 执行者角色
    pub role: String,
    /// 执行者类型 (e.g., codex, human, system)
    pub actor_type: String,
    /// 认证方式（执行报告必须非空）
    pub authentication_method: String,
}

/// M5 扩展的 WorkerReport（M5R01 精确执行 join 分型）
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct M5WorkerReport {
    /// 基础报告内容
    pub base: WorkerReport,
    /// 报告类型 (M5 分型；缺省 Manual，永不自动成为 Execution)
    pub kind: ReportKind,
    /// 执行回执 (仅 Execution 类型有)
    pub execution_receipt: Option<ExecutionReceipt>,
    /// 可信执行者
    pub actor: Option<TrustedActor>,
    /// 项目 ID
    pub project_id: Option<String>,
    /// 编排 ID
    pub orchestration_id: Option<String>,
    /// 工作流运行 ID（executed 必需；manual/offline 禁止）
    pub workflow_run_id: Option<String>,
    /// 工作项 ID（executed 必需；manual/offline 禁止）
    pub work_item_id: Option<String>,
    /// 节点 ID（executed 必需；manual/offline 禁止）
    pub node_id: Option<String>,
    /// 派发 ID（executed 必需；manual/offline 禁止）
    pub dispatch_id: Option<String>,
    /// 尝试 ID（executed 必需；manual/offline 禁止）
    pub attempt_id: Option<String>,
    /// Grant ID（executed 必需；manual/offline 禁止）
    pub grant_id: Option<String>,
    /// worker RoleSession ID（executed 必需；manual/offline 禁止）
    pub worker_role_session_id: Option<String>,
    /// authoritative receipt ref（executed 必需；manual/offline 禁止）
    pub authoritative_receipt_ref: Option<String>,
    /// 报告 hash（executed 必需）
    pub report_hash: Option<String>,
}

impl M5WorkerReport {
    /// 从基础 WorkerReport 创建；缺省分型为 Manual，不可自动成为执行报告
    pub(crate) fn from_base(base: WorkerReport) -> Self {
        Self {
            base,
            kind: ReportKind::Manual,
            execution_receipt: None,
            actor: None,
            project_id: None,
            orchestration_id: None,
            workflow_run_id: None,
            work_item_id: None,
            node_id: None,
            dispatch_id: None,
            attempt_id: None,
            grant_id: None,
            worker_role_session_id: None,
            authoritative_receipt_ref: None,
            report_hash: None,
        }
    }

    /// 设置为执行报告（必须显式输入回执与可信 actor）
    pub(crate) fn as_execution(mut self, receipt: ExecutionReceipt, actor: TrustedActor) -> Self {
        self.kind = ReportKind::Execution;
        self.execution_receipt = Some(receipt);
        self.actor = Some(actor);
        self
    }

    /// 设置为手动报告
    pub(crate) fn as_manual(mut self) -> Self {
        self.kind = ReportKind::Manual;
        self
    }

    /// 设置为离线报告
    pub(crate) fn as_offline(mut self) -> Self {
        self.kind = ReportKind::Offline;
        self
    }

    /// 绑定项目级上下文（manual/offline 允许；executed 也必需）
    pub(crate) fn bind_project(mut self, project_id: &str, orchestration_id: &str) -> Self {
        self.project_id = Some(project_id.to_string());
        self.orchestration_id = Some(orchestration_id.to_string());
        self
    }

    /// 绑定完整执行 join（仅 executed 使用）
    pub(crate) fn bind_execution_join(
        mut self,
        workflow_run_id: &str,
        work_item_id: &str,
        node_id: &str,
        dispatch_id: &str,
        attempt_id: &str,
        grant_id: &str,
        worker_role_session_id: &str,
        authoritative_receipt_ref: &str,
        report_hash: &str,
    ) -> Self {
        self.workflow_run_id = Some(workflow_run_id.to_string());
        self.work_item_id = Some(work_item_id.to_string());
        self.node_id = Some(node_id.to_string());
        self.dispatch_id = Some(dispatch_id.to_string());
        self.attempt_id = Some(attempt_id.to_string());
        self.grant_id = Some(grant_id.to_string());
        self.worker_role_session_id = Some(worker_role_session_id.to_string());
        self.authoritative_receipt_ref = Some(authoritative_receipt_ref.to_string());
        self.report_hash = Some(report_hash.to_string());
        self
    }

    /// 精确完整性核对（M5R01 合同 §2）：任何 ID 错配、缺 join、actor 自报或
    /// Grant 缺失都在业务写前 fail closed。
    pub(crate) fn verify_integrity(&self) -> Result<(), String> {
        match self.kind {
            ReportKind::Execution => {
                if self.execution_receipt.is_none() {
                    return Err("execution report missing receipt".to_string());
                }
                let Some(actor) = &self.actor else {
                    return Err("execution report missing trusted actor".to_string());
                };
                if actor.actor_id.trim().is_empty() || actor.authentication_method.trim().is_empty()
                {
                    return Err("execution report actor has no trusted authentication".to_string());
                }
                // actor 自报拒绝：执行者必须绑定 worker RoleSession
                if self.worker_role_session_id.is_none() {
                    return Err(
                        "execution report actor is not bound to a worker RoleSession".to_string(),
                    );
                }
                let required = [
                    ("project_id", &self.project_id),
                    ("orchestration_id", &self.orchestration_id),
                    ("workflow_run_id", &self.workflow_run_id),
                    ("work_item_id", &self.work_item_id),
                    ("node_id", &self.node_id),
                    ("dispatch_id", &self.dispatch_id),
                    ("attempt_id", &self.attempt_id),
                    ("grant_id", &self.grant_id),
                    ("worker_role_session_id", &self.worker_role_session_id),
                    ("authoritative_receipt_ref", &self.authoritative_receipt_ref),
                    ("report_hash", &self.report_hash),
                ];
                for (name, value) in required {
                    if value.as_ref().map(|v| v.trim().is_empty()).unwrap_or(true) {
                        return Err(format!(
                            "execution report missing exact join field: {}",
                            name
                        ));
                    }
                }
            }
            ReportKind::Manual | ReportKind::Offline => {
                // manual/offline：执行 join 字段必须缺席（M1 ManualOfflineClaim forbidden fields）
                let forbidden: [(&str, &Option<String>); 6] = [
                    ("dispatch_id", &self.dispatch_id),
                    ("attempt_id", &self.attempt_id),
                    ("grant_id", &self.grant_id),
                    ("worker_role_session_id", &self.worker_role_session_id),
                    ("authoritative_receipt_ref", &self.authoritative_receipt_ref),
                    ("workflow_run_id", &self.workflow_run_id),
                ];
                for (name, value) in forbidden {
                    if value.is_some() {
                        return Err(format!(
                            "manual/offline report must not carry execution join field: {}",
                            name
                        ));
                    }
                }
                if self.execution_receipt.is_some() {
                    return Err(
                        "manual/offline report must not carry execution receipt".to_string()
                    );
                }
                if self.project_id.is_none() || self.orchestration_id.is_none() {
                    return Err(
                        "manual/offline report missing project or orchestration context"
                            .to_string(),
                    );
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod m5_report_tests {
    use super::*;

    fn create_test_receipt() -> ExecutionReceipt {
        ExecutionReceipt {
            execution_id: "exec-1".to_string(),
            started_at_ms: 1000,
            completed_at_ms: Some(2000),
            status: "completed".to_string(),
            exit_code: Some(0),
            output_hash: Some("hash-123".to_string()),
            cost_tokens: Some(100),
        }
    }

    fn create_test_actor() -> TrustedActor {
        TrustedActor {
            actor_id: "codex-1".to_string(),
            role: "worker".to_string(),
            actor_type: "codex".to_string(),
            authentication_method: "m3_session_binding".to_string(),
        }
    }

    fn create_full_execution() -> M5WorkerReport {
        M5WorkerReport::from_base(WorkerReport::default())
            .as_execution(create_test_receipt(), create_test_actor())
            .bind_project("project-1", "orch-1")
            .bind_execution_join(
                "run-1",
                "item-1",
                "node-1",
                "dispatch-1",
                "attempt-1",
                "grant-1",
                "session-1",
                "receipt-ref-1",
                "hash-1",
            )
    }

    #[test]
    fn report_kind_display() {
        assert_eq!(ReportKind::Execution.to_string(), "executed");
        assert_eq!(ReportKind::Manual.to_string(), "manual");
        assert_eq!(ReportKind::Offline.to_string(), "offline");
    }

    #[test]
    fn report_kind_from_str() {
        assert_eq!(
            ReportKind::from_str("executed"),
            Some(ReportKind::Execution)
        );
        assert_eq!(
            ReportKind::from_str("execution"),
            Some(ReportKind::Execution)
        );
        assert_eq!(ReportKind::from_str("manual"), Some(ReportKind::Manual));
        assert_eq!(ReportKind::from_str("offline"), Some(ReportKind::Offline));
        assert_eq!(ReportKind::from_str("invalid"), None);
    }

    #[test]
    fn report_kind_checks() {
        assert!(ReportKind::Execution.is_execution());
        assert!(!ReportKind::Manual.is_execution());
        assert!(!ReportKind::Offline.is_execution());
        assert!(!ReportKind::Execution.is_manual_or_offline());
        assert!(ReportKind::Manual.is_manual_or_offline());
        assert!(ReportKind::Offline.is_manual_or_offline());
    }

    // RED PROBE: 缺省 ReportKind 不得自动成为执行报告（原候选缺口）
    #[test]
    fn default_report_kind_is_never_execution() {
        let m5_report = M5WorkerReport::from_base(WorkerReport::default());
        assert!(!m5_report.kind.is_execution());
        assert_eq!(m5_report.kind, ReportKind::Manual);
        // 未显式 as_execution 的输入缺字段必须失败
        assert!(m5_report.verify_integrity().is_err());
    }

    // RED PROBE: 执行报告缺少任一精确 join 字段即 fail closed
    #[test]
    fn execution_missing_workflow_run_id_fails() {
        let mut r = create_full_execution();
        r.workflow_run_id = None;
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_missing_work_item_id_fails() {
        let mut r = create_full_execution();
        r.work_item_id = None;
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_missing_node_id_fails() {
        let mut r = create_full_execution();
        r.node_id = None;
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_missing_dispatch_id_fails() {
        let mut r = create_full_execution();
        r.dispatch_id = None;
        assert!(r.verify_integrity().is_err());
    }

    // RED PROBE: Grant 缺失必须拒绝（原候选任意字符串 Grant 放行缺口）
    #[test]
    fn execution_missing_grant_id_fails() {
        let mut r = create_full_execution();
        r.grant_id = None;
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_missing_role_session_fails() {
        let mut r = create_full_execution();
        r.worker_role_session_id = None;
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_missing_receipt_ref_fails() {
        let mut r = create_full_execution();
        r.authoritative_receipt_ref = None;
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_missing_report_hash_fails() {
        let mut r = create_full_execution();
        r.report_hash = None;
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_missing_receipt_fails() {
        let mut r = create_full_execution();
        r.execution_receipt = None;
        assert!(r.verify_integrity().is_err());
    }

    // RED PROBE: actor 自报（有 actor 但无 worker RoleSession 绑定）拒绝
    #[test]
    fn execution_actor_without_role_session_fails() {
        let mut r = create_full_execution();
        r.worker_role_session_id = None;
        r.actor = Some(TrustedActor {
            actor_id: "self-reported".to_string(),
            role: "worker".to_string(),
            actor_type: "codex".to_string(),
            authentication_method: "self".to_string(),
        });
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn execution_actor_without_authentication_fails() {
        let mut r = create_full_execution();
        r.actor = Some(TrustedActor::default());
        assert!(r.verify_integrity().is_err());
    }

    // RED PROBE: manual/offline 携带执行 join 字段即拒绝
    #[test]
    fn manual_with_dispatch_join_fails() {
        let mut r = M5WorkerReport::from_base(WorkerReport::default())
            .as_manual()
            .bind_project("project-1", "orch-1");
        r.dispatch_id = Some("dispatch-1".to_string());
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn manual_with_grant_join_fails() {
        let mut r = M5WorkerReport::from_base(WorkerReport::default())
            .as_manual()
            .bind_project("project-1", "orch-1");
        r.grant_id = Some("grant-1".to_string());
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn manual_with_attempt_join_fails() {
        let mut r = M5WorkerReport::from_base(WorkerReport::default())
            .as_manual()
            .bind_project("project-1", "orch-1");
        r.attempt_id = Some("attempt-1".to_string());
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn offline_with_role_session_join_fails() {
        let mut r = M5WorkerReport::from_base(WorkerReport::default())
            .as_offline()
            .bind_project("project-1", "orch-1");
        r.worker_role_session_id = Some("session-1".to_string());
        assert!(r.verify_integrity().is_err());
    }

    #[test]
    fn manual_with_execution_receipt_fails() {
        let mut r = M5WorkerReport::from_base(WorkerReport::default())
            .as_manual()
            .bind_project("project-1", "orch-1");
        r.execution_receipt = Some(create_test_receipt());
        assert!(r.verify_integrity().is_err());
    }

    // POS: 完整精确 join 的执行报告通过
    #[test]
    fn execution_full_join_passes() {
        let r = create_full_execution();
        assert!(r.verify_integrity().is_ok());
    }

    // POS: manual/offline 只带项目上下文通过，永不冒充执行
    #[test]
    fn manual_project_only_passes() {
        let r = M5WorkerReport::from_base(WorkerReport::default())
            .as_manual()
            .bind_project("project-1", "orch-1");
        assert!(r.verify_integrity().is_ok());
        assert!(!r.kind.is_execution());
    }

    #[test]
    fn offline_project_only_passes() {
        let r = M5WorkerReport::from_base(WorkerReport::default())
            .as_offline()
            .bind_project("project-1", "orch-1");
        assert!(r.verify_integrity().is_ok());
    }
}
