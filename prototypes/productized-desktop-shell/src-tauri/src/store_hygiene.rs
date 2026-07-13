// Store 卫生维护：canvas-run 历史残料合法归档（ready_for_review → paused）。
//
// 任务包：tasks/2026-07-05-store-hygiene-canvas-run-residue-v1.md
//
// 主导线拍板「选项 A · 最小偏离」：canvas-run 残料挂在 `workflow:<slug>:<timestamp>`
// 这类 canvas 专属 workflow 下，而任务包点名要用、且要求 0-diff 的
// `update_work_item_state_at` 只认 `default_workflow_id(project_root)`（后缀写死 `:default`），
// 因此它 `find_work_item_index` 命中 0、够不到这批残料。此处按 work_item **自带的
// workflow_id** 定位，但迁移仍走 `control_core::validate_work_item_state_transition`
// 这道合法闸校验（ready_for_review → paused），并复用 `workflow_audit` 的标准审计构造器。
//
// 安全属性（与 update_work_item_state_at 等价，只是定位键不同）：
//   - 不删任何记录、不直接写 state 字段（先过合法闸再改）；
//   - 可逆：paused → ready_to_dispatch 是迁移表允许的一步（control_core.rs）；
//   - 迁移表 / control_core / update_work_item_state_at 全 0-diff（本文件只调用、不修改）；
//   - 默认 dry-run；写入前后 schema 校验 + 写前备份。

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

/// 超龄阈值：> 7 天（毫秒）。canvas-run 临时 work_item 一旦攒下即无人复审，
/// 7 天是保守下限——只归档明显陈旧的，绝不碰刚跑出来的。
const CANVAS_RUN_RESIDUE_MIN_AGE_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// work_item_id 里 canvas-run 形状的标记段（任务包判据：id 含 `:canvas-run:`）。
/// 交办形状（planned-task / 普通 work-item）不含此段，天然被排除。
const CANVAS_RUN_ID_MARKER: &str = ":canvas-run:";

const MILLIS_PER_DAY: i64 = 86_400_000;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SweepCanvasRunResidueRequest {
    /// 可选：限定某项目下的 canvas workflow（按 workflow_id 是否含该 project 的 slug 过滤）。
    /// None = 全库 canvas-run 残料。仅用于过滤，不参与迁移定位。
    #[serde(default)]
    pub(crate) project_root: Option<String>,
    /// 默认 true：只盘点、零写。
    #[serde(default = "default_true")]
    pub(crate) dry_run: bool,
    /// 前端传入的当前时间（毫秒），用于算岁数。与 work_item.created_at（毫秒串）同单位。
    pub(crate) now_ms: i64,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CanvasRunResidueItem {
    pub(crate) work_item_id: String,
    pub(crate) workflow_id: String,
    pub(crate) age_days: i64,
    /// execute 且成功迁移 = true；dry-run = false。
    pub(crate) swept: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SweepCanvasRunResidueResult {
    pub(crate) dry_run: bool,
    pub(crate) matched_count: usize,
    pub(crate) swept_count: usize,
    pub(crate) items: Vec<CanvasRunResidueItem>,
    /// execute 时的汇总审计 event_id。
    pub(crate) audit_event_id: Option<String>,
    pub(crate) backup_path: Option<String>,
    pub(crate) message: String,
}

/// slug 化 project_root，逻辑与 `plan_authorization_store::stable_id` 一致
/// （该 fn 为 mod-private 无法跨 mod 复用，这里内联一份仅用于 project_root 可选过滤）。
fn slugify_project_root(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// 判断一条 work_item 是否命中「canvas-run 形状 + ready_for_review + 超龄」三条件。
/// 命中返回 age_days；任一不满足（含无法解析 created_at）返回 None——不确定就不碰。
fn residue_age_days(item: &Value, now_ms: i64, project_slug: Option<&str>) -> Option<i64> {
    let work_item_id = crate::optional_string_from(item, "work_item_id")?;
    if !work_item_id.contains(CANVAS_RUN_ID_MARKER) {
        return None;
    }
    if crate::optional_string_from(item, "state").as_deref() != Some("ready_for_review") {
        return None;
    }
    let workflow_id = crate::optional_string_from(item, "workflow_id")?;
    if let Some(slug) = project_slug {
        if !workflow_id.contains(slug) {
            return None;
        }
    }
    let created_ms: i64 = crate::optional_string_from(item, "created_at")?
        .trim()
        .parse()
        .ok()?;
    let age_ms = now_ms - created_ms;
    if age_ms < CANVAS_RUN_RESIDUE_MIN_AGE_MS {
        return None;
    }
    Some(age_ms / MILLIS_PER_DAY)
}

#[tauri::command]
pub(crate) fn sweep_canvas_run_residue(
    request: SweepCanvasRunResidueRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<SweepCanvasRunResidueResult, String> {
    sweep_canvas_run_residue_at(&state.workflow_state_path, &request)
}

fn sweep_canvas_run_residue_at(
    path: &Path,
    request: &SweepCanvasRunResidueRequest,
) -> Result<SweepCanvasRunResidueResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法清理 canvas-run 残料".to_string());
    }

    let mut value = crate::read_workflow_state_value(path)?;
    let warnings = crate::validate_workflow_state(&value);
    if !warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            warnings.join(", ")
        ));
    }

    let project_slug = request.project_root.as_deref().map(slugify_project_root);

    // 盘点：收集命中项 (index, work_item_id, workflow_id, age_days)。只读。
    let mut matched: Vec<(usize, String, String, i64)> = Vec::new();
    if let Some(items) = value.get("work_items").and_then(Value::as_array) {
        for (idx, item) in items.iter().enumerate() {
            if let Some(age_days) = residue_age_days(item, request.now_ms, project_slug.as_deref())
            {
                let work_item_id =
                    crate::optional_string_from(item, "work_item_id").unwrap_or_default();
                let workflow_id =
                    crate::optional_string_from(item, "workflow_id").unwrap_or_default();
                matched.push((idx, work_item_id, workflow_id, age_days));
            }
        }
    }

    let threshold_days = CANVAS_RUN_RESIDUE_MIN_AGE_MS / MILLIS_PER_DAY;

    // dry-run（默认）：只返回盘点，零写。
    if request.dry_run {
        let items = matched
            .iter()
            .map(
                |(_, work_item_id, workflow_id, age_days)| CanvasRunResidueItem {
                    work_item_id: work_item_id.clone(),
                    workflow_id: workflow_id.clone(),
                    age_days: *age_days,
                    swept: false,
                },
            )
            .collect::<Vec<_>>();
        let matched_count = items.len();
        return Ok(SweepCanvasRunResidueResult {
            dry_run: true,
            matched_count,
            swept_count: 0,
            items,
            audit_event_id: None,
            backup_path: None,
            message: format!(
                "dry-run：找到 {matched_count} 条 canvas-run 历史残料（ready_for_review · 超 {threshold_days} 天），未做任何写入。"
            ),
        });
    }

    // execute 但无命中：不写、直接返回。
    if matched.is_empty() {
        return Ok(SweepCanvasRunResidueResult {
            dry_run: false,
            matched_count: 0,
            swept_count: 0,
            items: Vec::new(),
            audit_event_id: None,
            backup_path: None,
            message: "没有命中的 canvas-run 残料，未做任何写入。".to_string(),
        });
    }

    let timestamp = crate::unix_timestamp_string();

    let backup = crate::workflow_state_store::backup_file(path, &timestamp)?;

    // 逐条：合法闸校验 + 改 state + node 同步 + 标准 work_item_state_changed 审计。
    let mut result_items: Vec<CanvasRunResidueItem> = Vec::new();
    for (idx, work_item_id, workflow_id, age_days) in &matched {
        // 合法闸（逐条·复用 control_core，不改它）。before 恒为 ready_for_review。
        crate::control_core::validate_work_item_state_transition("ready_for_review", "paused")?;
        let node_id = crate::workflow_node_for_work_item_state(workflow_id, "paused");
        {
            let items = crate::array_mut(&mut value, "work_items")?;
            let item = items
                .get_mut(*idx)
                .ok_or_else(|| "work_items 索引越界；已中止（未提交）".to_string())?;
            // 二次确认 before，防处理途中被并发改写。
            if crate::optional_string_from(item, "state").as_deref() != Some("ready_for_review") {
                return Err("命中项状态在处理中被改变，已中止（未提交）".to_string());
            }
            item["state"] = Value::String("paused".to_string());
            item["current_node_id"] = Value::String(node_id.clone());
            item["updated_at"] = Value::String(timestamp.clone());
        }
        crate::update_node_state_for_id(&mut value, &node_id, "paused", &timestamp)?;

        let audit_event_id = format!(
            "audit:work-item-state:canvas-run-residue:{}:{timestamp}",
            slugify_project_root(work_item_id)
        );
        let audit = crate::workflow_audit::work_item_state_changed(
            crate::workflow_audit::WorkItemStateChangedAudit {
                event_id: audit_event_id,
                work_item_id: work_item_id.as_str(),
                before_state: "ready_for_review",
                after_state: "paused",
                created_at: timestamp.as_str(),
                reason:
                    "卫生：canvas-run 历史残料合法归档（ready_for_review → paused，可逆、不删）"
                        .to_string(),
            },
        );
        crate::array_mut(&mut value, "audit_events")?.push(audit);

        result_items.push(CanvasRunResidueItem {
            work_item_id: work_item_id.clone(),
            workflow_id: workflow_id.clone(),
            age_days: *age_days,
            swept: true,
        });
    }

    // 汇总审计 canvas_run_residue_swept（手构；不碰 workflow_audit.rs——它不在文件面）。
    let swept_count = result_items.len();
    let summary_event_id = format!("audit:canvas-run-residue-swept:{timestamp}");
    let summary_audit = json!({
        "event_id": summary_event_id,
        "event_type": "canvas_run_residue_swept",
        "target_ref": "canvas_run_residue",
        "actor_ref": "user_confirmed_desktop_shell",
        "source_kind": "workspace_state",
        "permission_level": "user_confirmed_write",
        "swept_count": swept_count,
        "created_at": timestamp,
        "reason": format!(
            "卫生：归档 {swept_count} 条 canvas-run 历史残料（ready_for_review → paused，合法可逆、不删任何记录）。"
        ),
    });
    crate::array_mut(&mut value, "audit_events")?.push(summary_audit);

    value["updated_at"] = Value::String(timestamp.clone());

    // 写前 schema 校验。
    let warnings = crate::validate_workflow_state(&value);
    if !warnings.is_empty() {
        return Err(format!("写入前 schema 校验失败：{}", warnings.join(", ")));
    }
    crate::write_validated_workflow_state(path, &value)?;

    // 写后回读校验。
    let snapshot = crate::read_workflow_state_snapshot(path)?;
    if !snapshot.exists {
        return Err("归档写入后重新读取校验失败".to_string());
    }

    Ok(SweepCanvasRunResidueResult {
        dry_run: false,
        matched_count: matched.len(),
        swept_count,
        items: result_items,
        audit_event_id: Some(summary_event_id),
        backup_path: Some(backup.display().to_string()),
        message: format!(
            "已归档 {swept_count} 条 canvas-run 历史残料（ready_for_review → paused，可逆）。已写前备份旧状态。"
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const NOW_MS: i64 = 1_783_000_000_000; // 固定「现在」，测试确定性
    const OLD_MS: i64 = NOW_MS - 10 * MILLIS_PER_DAY; // 超龄 10 天
    const FRESH_MS: i64 = NOW_MS - 1 * MILLIS_PER_DAY; // 未超龄 1 天

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        fs::create_dir_all(&dir).expect("test dir");
        dir
    }

    fn work_item(id: &str, workflow_id: &str, state: &str, created_ms: i64) -> Value {
        json!({
            "work_item_id": id,
            "workflow_id": workflow_id,
            "state": state,
            "current_node_id": format!("{workflow_id}:node:review"),
            "created_at": created_ms.to_string(),
            "updated_at": created_ms.to_string(),
        })
    }

    /// 一个通过 validate_workflow_state 的最小合法 store，塞入各类 work_item。
    fn fixture_state(work_items: Vec<Value>) -> Value {
        json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": "seed",
            "projects": [],
            "agent_adapters": [],
            "workflows": [],
            "nodes": [],
            "edges": [],
            "work_items": work_items,
            "artifacts": [],
            "reviews": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": []
        })
    }

    fn write_fixture(dir: &Path, work_items: Vec<Value>) -> PathBuf {
        let path = dir.join("workflow-state.v0.json");
        let text = serde_json::to_string_pretty(&fixture_state(work_items)).unwrap();
        fs::write(&path, text).expect("write fixture");
        path
    }

    // 三类应命中 + 四类不该碰的混合样本。
    fn mixed_items() -> Vec<Value> {
        let cwf = "workflow:users-yoyi-codex-workflow-mario-test:1782115268646";
        vec![
            // 命中：canvas-run + ready_for_review + 超龄（×2）
            work_item(
                &format!("work-item:{cwf}:canvas-run:1"),
                cwf,
                "ready_for_review",
                OLD_MS,
            ),
            work_item(
                &format!("work-item:{cwf}:canvas-run:2"),
                cwf,
                "ready_for_review",
                OLD_MS,
            ),
            // 不碰：canvas-run 但未超龄
            work_item(
                &format!("work-item:{cwf}:canvas-run:3"),
                cwf,
                "ready_for_review",
                FRESH_MS,
            ),
            // 不碰：canvas-run 但非 ready_for_review
            work_item(
                &format!("work-item:{cwf}:canvas-run:4"),
                cwf,
                "running",
                OLD_MS,
            ),
            // 不碰：交办形状（planned-task，非 canvas-run id）+ ready_for_review + 超龄
            work_item(
                "work-item:workflow:users-yoyi-codex-workflow-mario-test:default:planned-task:x",
                "workflow:users-yoyi-codex-workflow-mario-test:default",
                "ready_for_review",
                OLD_MS,
            ),
            // 不碰：普通交办 work-item + ready_for_review + 超龄
            work_item(
                "work-item:workflow:users-yoyi-codex-workflow-mario-test:default:1",
                "workflow:users-yoyi-codex-workflow-mario-test:default",
                "ready_for_review",
                OLD_MS,
            ),
        ]
    }

    fn req(dry_run: bool) -> SweepCanvasRunResidueRequest {
        SweepCanvasRunResidueRequest {
            project_root: None,
            dry_run,
            now_ms: NOW_MS,
        }
    }

    #[test]
    fn dry_run_reports_only_matches_and_writes_nothing() {
        let dir = temp_dir("sweep-dry");
        let path = write_fixture(&dir, mixed_items());
        let before = fs::read(&path).unwrap();

        let result = sweep_canvas_run_residue_at(&path, &req(true)).expect("dry-run ok");

        assert!(result.dry_run);
        assert_eq!(result.matched_count, 2, "只该命中 2 条 canvas-run 超龄残料");
        assert_eq!(result.swept_count, 0);
        assert!(result.items.iter().all(|i| !i.swept));
        // 零写：字节完全不变
        assert_eq!(fs::read(&path).unwrap(), before, "dry-run 必须零写");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn execute_migrates_only_matches_to_paused_with_audit() {
        let dir = temp_dir("sweep-exec");
        let path = write_fixture(&dir, mixed_items());

        let result = sweep_canvas_run_residue_at(&path, &req(false)).expect("execute ok");
        assert_eq!(result.swept_count, 2);
        assert!(result.audit_event_id.is_some());
        assert!(result.backup_path.is_some());

        let after: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let items = after["work_items"].as_array().unwrap();
        let state_of = |id_suffix: &str| -> String {
            items
                .iter()
                .find(|w| w["work_item_id"].as_str().unwrap().ends_with(id_suffix))
                .and_then(|w| w["state"].as_str())
                .unwrap()
                .to_string()
        };
        // 命中的 2 条 → paused
        assert_eq!(state_of(":canvas-run:1"), "paused");
        assert_eq!(state_of(":canvas-run:2"), "paused");
        // 未超龄 / 非 ready_for_review 的 canvas-run → 不动
        assert_eq!(state_of(":canvas-run:3"), "ready_for_review");
        assert_eq!(state_of(":canvas-run:4"), "running");
        // 交办形状 → 不动
        assert_eq!(state_of(":planned-task:x"), "ready_for_review");
        assert_eq!(state_of(":default:1"), "ready_for_review");

        // 审计：2 条标准 + 1 条汇总
        let audits = after["audit_events"].as_array().unwrap();
        let per_item = audits
            .iter()
            .filter(|a| a["event_type"] == "work_item_state_changed")
            .count();
        let summary = audits
            .iter()
            .filter(|a| a["event_type"] == "canvas_run_residue_swept")
            .count();
        assert_eq!(per_item, 2, "每条命中一条标准迁移审计");
        assert_eq!(summary, 1, "一条汇总审计");
        assert_eq!(
            audits
                .iter()
                .find(|a| a["event_type"] == "canvas_run_residue_swept")
                .unwrap()["swept_count"],
            2
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn paused_is_reversible_per_migration_table() {
        // 归档用的 paused 是合法可逆态：paused → ready_to_dispatch 必须被迁移表允许。
        assert!(crate::control_core::validate_work_item_state_transition(
            "ready_for_review",
            "paused"
        )
        .is_ok());
        assert!(crate::control_core::validate_work_item_state_transition(
            "paused",
            "ready_to_dispatch"
        )
        .is_ok());
    }

    #[test]
    fn execute_is_idempotent() {
        let dir = temp_dir("sweep-idem");
        let path = write_fixture(&dir, mixed_items());

        let first = sweep_canvas_run_residue_at(&path, &req(false)).expect("first ok");
        assert_eq!(first.swept_count, 2);
        // 重跑：命中项已 paused、不再 ready_for_review → 0 命中、0 迁移。
        let second = sweep_canvas_run_residue_at(&path, &req(false)).expect("second ok");
        assert_eq!(second.matched_count, 0);
        assert_eq!(second.swept_count, 0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_root_filter_scopes_to_matching_workflow() {
        let dir = temp_dir("sweep-scope");
        let other = "workflow:users-yoyi-other-project:1782115268646";
        let mut items = mixed_items();
        // 另一个项目的 canvas-run 超龄残料——带 project_root 过滤时不该被扫到。
        items.push(work_item(
            &format!("work-item:{other}:canvas-run:9"),
            other,
            "ready_for_review",
            OLD_MS,
        ));
        let path = write_fixture(&dir, items);

        let scoped = SweepCanvasRunResidueRequest {
            project_root: Some("/Users/yoyi/codex-workflow-mario-test".to_string()),
            dry_run: true,
            now_ms: NOW_MS,
        };
        let result = sweep_canvas_run_residue_at(&path, &scoped).expect("scoped ok");
        assert_eq!(result.matched_count, 2, "只扫 mario-test 项目下的 2 条");
        let _ = fs::remove_dir_all(dir);
    }
}
