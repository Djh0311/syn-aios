// B·审计账本读模型（新「审计账本页」的数据）·**纯只读聚合**。
//
// 任务包：tasks/2026-07-15-backend-ui-support-readmodels-package-v1.md §B
//
// 定位：把主 store 与各 sidecar 的审计**拼成一条按时间倒序的统一流**，分页 + 按类过滤。
// **零写入·零新表·零改写路·零 LM**；只调各店现成的只读 loader（与 reconcile 同一批读法）。
//
// 读法（照 reconcile `reconcile_db_vs_json`:350 的现成口径复用·不另造）：
//   · 主 store        = `read_workflow_state_value` → `audit_events`
//   · 方案            = `project_consultation_proposal_store::load_store` → `audit_events`
//   · 授权            = `plan_authorization_store::load_store` → `audit_events`
//   · 主管编排        = `mcp::supervisor_orchestrator::db_primary_projection_records` → 第二项
//   · 全局主管复核    = `global_supervisor_review_store::load_store` → 两半（结果复核 + 批前边界）
//   · 受控续话        = `session_continuation_store::load_store` → `audit_events`
//
// **模式与读源的诚实说明**（任务包写「db_primary 下从 DB 读、json_only 从 JSON 读」，这里按事实收敛）：
// db_primary 是 **lag=0 的 JSON 投影**——JSON 不是缓存副本而是同笔事务的投影，DB 与 JSON 恒等
// （不等即启动对账 fail-closed、根本起不来）。故两模式都走上面这批 JSON loader **读到的是同一份事实**，
// 且能顺带白拿各店 loader 自带的 schema 容错。另有一条硬事实：降级审计
// `storage_mode_degraded_json_only` **只写 JSON 不写 DB**（写它时 DB 主写已冻），DB 侧根本没有它——
// 走 DB 读反而会漏掉降级记录。`storage_mode` 字段如实回传，前端要显示读源可直接用。
// （若将来 M6 停写 JSON，本模块的读源必须整体改走 DB——已在回传里挂账，不是遗漏。）
//
// 字段映射：各店审计**形状本就不同**（主 store `event_id`+`created_at` 毫秒串；方案/授权
// `audit_event_id`+`created_at_ms` i64；主管编排 `tool`+`result_summary` 没有 event_type/reason……）。
// 这里不硬编码 N 套映射，而是按**显式优先级**取第一个存在的字段（见 `AUDIT_*_KEYS`）——
// 加一个新 sidecar 时多半零改动就能进流，取不到也只是降级成保守值、不炸。
//
// 红线：只读聚合·不新增表/不改写路·不碰安全闸与存储模式语义；软着陆（某店缺失/损坏 → warnings +
// 跳过该源，不 Err 断整页）。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

/// 事件 id 字段的取值优先级（首个存在者胜）。
const AUDIT_ID_KEYS: &[&str] = &["event_id", "audit_event_id"];
/// 事件类型字段：`tool` 是主管编排审计的等价位（它没有 event_type）。
const AUDIT_TYPE_KEYS: &[&str] = &["event_type", "tool"];
/// 人话摘要字段：`reason` 是绝大多数店的人话位；`result_summary` 是主管编排的等价位。
/// 全缺 → 回落 event_type（任务包口径：「有则用现有人话字段,无则 event_type」）。
const AUDIT_SUMMARY_KEYS: &[&str] = &["reason", "result_summary", "summary"];
/// 归属对象字段：按「越具体越优先」排。
const AUDIT_TARGET_KEYS: &[&str] = &[
    "target_ref",
    "proposal_id",
    "authorization_id",
    "continuation_id",
    "work_item_id",
    "workflow_id",
    "run_id",
];

const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 500;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AuditLedgerQuery {
    /// 0 基页码；越界 → 空 items（total 照报·前端好回跳）。
    #[serde(default)]
    pub(crate) page: usize,
    #[serde(default)]
    pub(crate) page_size: Option<usize>,
    /// 按 `event_type` 精确过滤；None/空 = 不过滤。
    #[serde(default)]
    pub(crate) kind_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuditLedgerItem {
    pub(crate) at_ms: i64,
    /// 来源店名（稳定机器键·UI 自己映射人话）。
    pub(crate) source: String,
    pub(crate) event_type: String,
    pub(crate) human_summary: String,
    pub(crate) target_ref: Option<String>,
    pub(crate) raw_json: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuditLedgerPage {
    /// **过滤后**的总条数（不是本页条数·前端「共 N 条」用）。
    pub(crate) total: usize,
    pub(crate) items: Vec<AuditLedgerItem>,
    pub(crate) page: usize,
    pub(crate) page_size: usize,
    /// 本页数据实际读自哪种模式："db_primary" | "json_only"（读源说明见文件头）。
    pub(crate) storage_mode: String,
    /// 过滤前流里出现过的全部 event_type（升序去重）——前端过滤下拉直接用，不用自己猜有哪些类。
    pub(crate) kinds: Vec<String>,
    pub(crate) warnings: Vec<String>,
}

#[tauri::command]
pub(crate) fn query_audit_ledger_read_model(
    request: AuditLedgerQuery,
    state: tauri::State<'_, crate::AppState>,
) -> AuditLedgerPage {
    query_audit_ledger_read_model_at(&state.workflow_state_path, &request)
}

pub(crate) fn query_audit_ledger_read_model_at(
    workflow_state_path: &Path,
    request: &AuditLedgerQuery,
) -> AuditLedgerPage {
    let mut warnings = Vec::new();
    let storage_mode =
        match crate::workbench_sqlite_storage_mode::storage_mode_for(workflow_state_path) {
            crate::workbench_sqlite_storage_mode::StorageMode::JsonOnly { .. } => "json_only",
            crate::workbench_sqlite_storage_mode::StorageMode::DbPrimaryJsonProjection(_) => {
                "db_primary"
            }
        }
        .to_string();

    let mut items = collect_all_sources(workflow_state_path, &mut warnings);
    // 时间倒序（新的在前）；同毫秒按 source+id 稳定兜底，保证翻页不抖。
    items.sort_by(|left, right| {
        right
            .at_ms
            .cmp(&left.at_ms)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.event_type.cmp(&right.event_type))
    });

    let mut kinds: Vec<String> = items.iter().map(|item| item.event_type.clone()).collect();
    kinds.sort();
    kinds.dedup();

    let filtered: Vec<AuditLedgerItem> = match request
        .kind_filter
        .as_deref()
        .map(str::trim)
        .filter(|filter| !filter.is_empty())
    {
        Some(filter) => items
            .into_iter()
            .filter(|item| item.event_type == filter)
            .collect(),
        None => items,
    };

    let total = filtered.len();
    let page_size = request
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let page_items = filtered
        .into_iter()
        .skip(request.page.saturating_mul(page_size))
        .take(page_size)
        .collect();

    AuditLedgerPage {
        total,
        items: page_items,
        page: request.page,
        page_size,
        storage_mode,
        kinds,
        warnings,
    }
}

/// 逐源装配。任一源坏了只记 warning 并跳过该源——别让一个 sidecar 炸掉整页。
fn collect_all_sources(workflow_state_path: &Path, warnings: &mut Vec<String>) -> Vec<AuditLedgerItem> {
    let mut items = Vec::new();

    match crate::read_workflow_state_value(workflow_state_path) {
        Ok(value) => match value.get("audit_events").and_then(Value::as_array) {
            Some(events) => push_source(&mut items, "workflow_state", events.iter().cloned()),
            None => warnings.push("主 store 没有 audit_events 数组，这一段审计没进流。".to_string()),
        },
        Err(error) => warnings.push(format!("主 store 读不了，这一段审计没进流：{error}")),
    }

    let now_ms = crate::unix_timestamp_ms();

    match crate::project_consultation_proposal_store::load_store(workflow_state_path, now_ms) {
        Ok(store) => push_serializable_source(
            &mut items,
            "project_consultation_proposal",
            store.audit_events,
            warnings,
        ),
        Err(error) => warnings.push(format!("方案店读不了，这一段审计没进流：{error}")),
    }

    match crate::plan_authorization_store::load_store(workflow_state_path, now_ms) {
        Ok(store) => push_serializable_source(
            &mut items,
            "plan_authorization",
            store.audit_events,
            warnings,
        ),
        Err(error) => warnings.push(format!("授权店读不了，这一段审计没进流：{error}")),
    }

    match crate::mcp::supervisor_orchestrator::db_primary_projection_records(workflow_state_path) {
        Ok((_sessions, audit_events)) => {
            push_source(&mut items, "supervisor_orchestrator", audit_events.into_iter())
        }
        Err(error) => warnings.push(format!("主管编排店读不了，这一段审计没进流：{error}")),
    }

    // 本店只有 soft loader（缺失/坏 → 空 store + 人话 warnings），正合读模型口径：照收它的 warnings。
    let (review_store, review_warnings) =
        crate::global_supervisor_review_store::load_store_soft(workflow_state_path, now_ms);
    warnings.extend(review_warnings);
    push_serializable_source(
        &mut items,
        "global_supervisor_review",
        review_store.audit_events,
        warnings,
    );
    push_serializable_source(
        &mut items,
        "global_supervisor_boundary_review",
        review_store.boundary_audit_events,
        warnings,
    );

    match crate::session_continuation_store::load_store(
        workflow_state_path,
        &crate::unix_timestamp_string(),
    ) {
        Ok(store) => push_serializable_source(
            &mut items,
            "session_continuation",
            store.audit_events,
            warnings,
        ),
        Err(error) => warnings.push(format!("受控续话店读不了，这一段审计没进流：{error}")),
    }

    items
}

fn push_source(
    items: &mut Vec<AuditLedgerItem>,
    source: &str,
    events: impl Iterator<Item = Value>,
) {
    for event in events {
        items.push(map_event(source, event));
    }
}

/// 结构化审计 → Value 再走同一套映射（各店结构体私有·统一按 Value 取字段，不为读模型改任何店的可见性）。
fn push_serializable_source<T: Serialize>(
    items: &mut Vec<AuditLedgerItem>,
    source: &str,
    events: Vec<T>,
    warnings: &mut Vec<String>,
) {
    for event in events {
        match serde_json::to_value(event) {
            Ok(value) => items.push(map_event(source, value)),
            Err(error) => warnings.push(format!("{source} 的一条审计序列化失败，已跳过：{error}")),
        }
    }
}

fn map_event(source: &str, event: Value) -> AuditLedgerItem {
    let event_type = first_string(&event, AUDIT_TYPE_KEYS).unwrap_or_else(|| "unknown".to_string());
    let human_summary = first_non_empty_string(&event, AUDIT_SUMMARY_KEYS)
        // 任务包口径：没有人话字段就用 event_type 顶上（不编人话）。
        .unwrap_or_else(|| event_type.clone());
    AuditLedgerItem {
        at_ms: event_at_ms(&event).unwrap_or(0),
        source: source.to_string(),
        event_type,
        human_summary,
        target_ref: first_non_empty_string(&event, AUDIT_TARGET_KEYS),
        raw_json: event,
    }
}

/// 时间戳两种形状都认：`created_at_ms`(i64) 与 `created_at`(毫秒串)。
/// 都读不出 → None → 调用方按 0 处理（排最后·不假装有时间）。
fn event_at_ms(event: &Value) -> Option<i64> {
    if let Some(at_ms) = event.get("created_at_ms").and_then(Value::as_i64) {
        return Some(at_ms);
    }
    if let Some(created_at) = event.get("created_at").and_then(Value::as_str) {
        if let Ok(at_ms) = created_at.trim().parse::<i64>() {
            return Some(at_ms);
        }
    }
    None
}

fn first_string(event: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        event
            .get(*key)
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

fn first_non_empty_string(event: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        event
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

/// 未用的 id 取值口径先留着：`raw_json` 已带全字段，UI 要 id 自己取；
/// 但保留常量清单是为了「加新 sidecar 时照这张表对字段」的可读性。
#[allow(dead_code)]
fn event_id(event: &Value) -> Option<String> {
    first_non_empty_string(event, AUDIT_ID_KEYS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::UNIX_EPOCH;

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TempRoot(PathBuf);

    impl Drop for TempRoot {
        fn drop(&mut self) {
            crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
                &self.0.join("workflow-state").join("workflow-state.v0.json"),
            );
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const BASE_MS: i64 = 1_700_000_000_000;

    fn fixture(label: &str) -> (TempRoot, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "audit-ledger-read-model-{label}-{}-{}",
            crate::unix_timestamp_nanos(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&root).expect("create temp root");
        let root = fs::canonicalize(&root).expect("canonical temp root");
        let project_root = root.join("project");
        fs::create_dir_all(&project_root).expect("create fixture project root");
        let state_path = root.join("workflow-state").join("workflow-state.v0.json");
        fs::create_dir_all(state_path.parent().expect("state parent")).expect("state parent");
        crate::bootstrap_project_workflow_at(
            &state_path,
            &crate::ProjectRecord {
                project_root: project_root.display().to_string(),
                name: "audit ledger fixture".to_string(),
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
            },
        )
        .expect("bootstrap workflow state");
        (TempRoot(root), state_path)
    }

    fn push_main_audit(state_path: &Path, event_type: &str, at_ms: i64, reason: &str) {
        let mut value = crate::read_workflow_state_value(state_path).expect("read state");
        value["audit_events"]
            .as_array_mut()
            .expect("audit_events array")
            .push(json!({
                "event_id": format!("{event_type}:{at_ms}"),
                "event_type": event_type,
                "target_ref": "wf-1",
                "actor_ref": "tester",
                "created_at": at_ms.to_string(),
                "reason": reason,
            }));
        fs::write(
            state_path,
            serde_json::to_string_pretty(&value).expect("serialize state"),
        )
        .expect("write state");
    }

    fn query(page: usize, page_size: Option<usize>, kind_filter: Option<&str>) -> AuditLedgerQuery {
        AuditLedgerQuery {
            page,
            page_size,
            kind_filter: kind_filter.map(str::to_string),
        }
    }

    fn mtime(path: &Path) -> u128 {
        fs::metadata(path)
            .expect("state metadata")
            .modified()
            .expect("state mtime")
            .duration_since(UNIX_EPOCH)
            .expect("mtime after epoch")
            .as_nanos()
    }

    // 端到端：造数据 → 命令 → 形状。含倒序、人话字段、kinds 汇总。
    #[test]
    fn aggregates_main_store_audits_newest_first_with_human_summary() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("aggregate");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );
        push_main_audit(&state_path, "older_event", BASE_MS, "早一点那件事。");
        push_main_audit(&state_path, "newer_event", BASE_MS + 1_000, "晚一点那件事。");

        let page = query_audit_ledger_read_model_at(&state_path, &query(0, None, None));

        let ours: Vec<&AuditLedgerItem> = page
            .items
            .iter()
            .filter(|item| item.event_type.ends_with("_event"))
            .collect();
        assert_eq!(ours.len(), 2, "两条都该进流：{:?}", page.items);
        assert_eq!(ours[0].event_type, "newer_event", "时间倒序·新的在前");
        assert_eq!(ours[0].source, "workflow_state");
        assert_eq!(ours[0].human_summary, "晚一点那件事。", "有 reason 就用 reason");
        assert_eq!(ours[0].target_ref.as_deref(), Some("wf-1"));
        assert_eq!(ours[0].at_ms, BASE_MS + 1_000, "毫秒串 created_at 要解析成 at_ms");
        assert!(
            page.kinds.iter().any(|kind| kind == "newer_event"),
            "kinds 要汇总出现过的类：{:?}",
            page.kinds
        );
        assert_eq!(page.storage_mode, "json_only");
        // raw_json 原样带全字段，前端下钻用。
        assert_eq!(ours[0].raw_json["actor_ref"], json!("tester"));
        // 各 sidecar 没建 = 正常态（loader 返回空店），**不许**因此刷 warning——
        // 否则每次翻页都给用户一脸「读不了」噪音。
        assert!(
            page.warnings.is_empty(),
            "sidecar 缺席是正常态·不该出 warning：{:?}",
            page.warnings
        );
    }

    // 没有人话字段 → 回落 event_type（任务包口径：不编人话）。
    #[test]
    fn event_without_human_field_falls_back_to_event_type() {
        let item = map_event(
            "supervisor_orchestrator",
            json!({"event_id": "e1", "tool": "dispatch_worker", "created_at_ms": BASE_MS}),
        );
        assert_eq!(item.event_type, "dispatch_worker", "tool 是编排审计的 event_type 等价位");
        assert_eq!(item.human_summary, "dispatch_worker", "没人话字段就用 event_type 顶上");
        assert_eq!(item.at_ms, BASE_MS);
    }

    // 主管编排审计：result_summary 是它的人话位。
    #[test]
    fn orchestrator_result_summary_is_used_as_human_summary() {
        let item = map_event(
            "supervisor_orchestrator",
            json!({
                "event_id": "e2",
                "tool": "follow_up_worker",
                "result_summary": "已把追问发给 worker。",
                "run_id": "run-9",
                "created_at_ms": BASE_MS,
            }),
        );
        assert_eq!(item.human_summary, "已把追问发给 worker。");
        assert_eq!(item.target_ref.as_deref(), Some("run-9"), "没 target_ref 就退到 run_id");
    }

    // 分页 + 过滤：total 是**过滤后**总数，不是本页条数。
    #[test]
    fn filter_and_pagination_report_filtered_total() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("paging");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );
        for index in 0..5 {
            push_main_audit(
                &state_path,
                "wanted_kind",
                BASE_MS + index * 1_000,
                &format!("第 {index} 件"),
            );
        }
        push_main_audit(&state_path, "other_kind", BASE_MS + 99_000, "别的类。");

        let first = query_audit_ledger_read_model_at(
            &state_path,
            &query(0, Some(2), Some("wanted_kind")),
        );
        assert_eq!(first.total, 5, "total = 过滤后总数");
        assert_eq!(first.items.len(), 2, "本页 2 条");
        assert_eq!(first.page_size, 2);
        assert!(first.items.iter().all(|item| item.event_type == "wanted_kind"));

        let last = query_audit_ledger_read_model_at(
            &state_path,
            &query(2, Some(2), Some("wanted_kind")),
        );
        assert_eq!(last.items.len(), 1, "第 3 页余 1 条");

        let beyond = query_audit_ledger_read_model_at(
            &state_path,
            &query(99, Some(2), Some("wanted_kind")),
        );
        assert!(beyond.items.is_empty(), "越界页 → 空 items");
        assert_eq!(beyond.total, 5, "越界页仍如实报 total·前端好回跳");
    }

    // 读模型的命门：**不许写盘**。
    #[test]
    fn read_model_never_writes_workflow_state() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("read-only");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );
        push_main_audit(&state_path, "some_event", BASE_MS, "一件事。");
        let before = mtime(&state_path);

        let _ = query_audit_ledger_read_model_at(&state_path, &query(0, None, None));
        let _ = query_audit_ledger_read_model_at(&state_path, &query(0, None, None));

        assert_eq!(before, mtime(&state_path), "读模型纯只读·不许写 workflow state");
    }

    // 某源坏了只跳过该源 + 出 warning，不炸整页（其余源照出）。
    #[test]
    fn broken_main_store_soft_lands_without_breaking_the_page() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("broken");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );
        fs::write(&state_path, "{ not json").expect("corrupt state");

        let page = query_audit_ledger_read_model_at(&state_path, &query(0, None, None));

        assert!(
            page.warnings.iter().any(|warning| warning.contains("主 store 读不了")),
            "坏源要有人话报备：{:?}",
            page.warnings
        );
    }
}
