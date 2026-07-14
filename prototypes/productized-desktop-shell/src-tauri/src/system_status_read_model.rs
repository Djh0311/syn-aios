// A·系统状态读模型（首页「系统状态」区块 + 顶栏健康点）·**纯只读**。
//
// 任务包：tasks/2026-07-15-backend-ui-support-readmodels-package-v1.md §A
//
// 定位：把「存储模式 / DB 主写健康 / 观察期第几天 / 最近一次降级 / 写解封闸」这几件散在后端的
// 运行事实，拼成首页一屏能直接接的形状。**零写入·零审计·零 LM·零新写点**；存储模式语义一字不碰
// （只调新增的只读访问器 `db_primary_health_snapshot`：不建缓存条目、不改健康态、不触发对账）。
//
// 数据事实（勘察逐条核过·坐标见回传）：
//   · 模式   = `workbench_sqlite_storage_mode::storage_mode_for`（配置 fail-closed·按路径缓存·
//              进程内不热切）。**复用现成访问器**，不自己再解析一遍 storage-mode 配置（防两份判据分叉）。
//   · 健康   = 同模块 `health_cache`：Ready / Blocked(reason) / **None = 本进程没跑过启动对账**。
//              降级**不改模式**（模式缓存仍是 db_primary），只把健康翻 Blocked、写落回 JSON——
//              故「配置 db_primary + healthy=false」才是降级态的真实长相（见 `last_degradation`）。
//   · 观察期起点 = 主 store `audit_events` 里**最新**一条 `storage_mode_initialized`（每次 apply/重种
//              都会再写一条·`append_startup_mode_audit`:664·真机已 17 条）。「重开」口径＝07-14 拍板
//              「重 seed → 观察期重开」：任何再初始化都重开观察期，对 M6 提案保守。
//              （任务包原文写「首条」是包文本过期，总指导核验时按已拍口径改；歧义记录见 handoff §5②。）
//   · 最近降级 = **最晚**一条 `storage_mode_degraded_json_only`（`append_blocked_json_only_degradation_audit`
//              :705 写）。该事件**只写 JSON 不写 DB**（写它时 DB 主写已冻），故 JSON 主 store 是它唯一真源——
//              两模式下都读 JSON 是对的，不是偷懒。其 `reason` 后端已写成人话，直接上脸不再包一层。
//   · 写解封闸 = `commands.rs` 编译期常量（站 3b 只读 / 站 4 写·仅 mario test）——运行时无配置可改，
//              故摘要由常量派生（跟着常量走·不会漂）。
//
// 红线：软着陆——任何缺失/坏/判不了 → `warnings` + 保守值，**不 Err 断首页**（增益不是闸）；
// 不新增写点、不改任何写入路径、不碰安全闸与解封面。

use serde::Serialize;
use std::path::Path;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
const EVENT_STORAGE_MODE_INITIALIZED: &str = "storage_mode_initialized";
const EVENT_STORAGE_MODE_DEGRADED: &str = "storage_mode_degraded_json_only";
/// 最近拦截条数上限（形状占位·见 `recent_catches` 的说明）。
const RECENT_CATCH_LIMIT: usize = 5;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SystemStatusDegradation {
    pub(crate) at_ms: i64,
    /// 后端降级审计里的 `reason`（已是人话）；缺则给一句保守兜底。
    pub(crate) reason_human: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SystemStatusCatch {
    pub(crate) at_ms: i64,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SystemStatusReadModel {
    /// **配置**模式（非「此刻实际写哪」）："db_primary" | "json_only"。
    /// 降级时这里仍是 db_primary、而 `storage_healthy=false`——两个字段合起来才是全貌。
    pub(crate) storage_mode: String,
    /// db_primary：启动对账绿 = true；Blocked 或**没跑过启动对账** = false。
    /// json_only：配置如此即正常运行 = true。
    pub(crate) storage_healthy: bool,
    /// 观察期第 N 天（切换当天 = 第 1 天）。无 `storage_mode_initialized` 审计（= 没进过观察期）→ 0。
    pub(crate) observation_day: u32,
    pub(crate) last_degradation: Option<SystemStatusDegradation>,
    /// 最近拦截。**当前恒空**：拦截账本是 `docs/harness-catch-log.md`（人手维护的开发流程档案·
    /// 在仓里不在 app 数据根），运行时没有对应数据源；app 去读仓内 md 属越界，故先留形状回空数组 +
    /// 一条 warning 说明。要真填需另立「运行时拦截事件」写点（本包只读·不新增写点）。
    pub(crate) recent_catches: Vec<SystemStatusCatch>,
    /// 写解封闸一句话（如「mario test 写解封·仅此项目」）。
    pub(crate) gate_summary: Option<String>,
    /// 软着陆报备（人话·不断面板）。
    pub(crate) warnings: Vec<String>,
}

#[tauri::command]
pub(crate) fn load_system_status_read_model(
    state: tauri::State<'_, crate::AppState>,
) -> SystemStatusReadModel {
    // 纯只读·同步（读盘装配快·无 LM·无需 spawn_blocking），与 list_project_run_history 同款。
    load_system_status_read_model_at(&state.workflow_state_path, crate::unix_timestamp_ms())
}

pub(crate) fn load_system_status_read_model_at(
    workflow_state_path: &Path,
    now_ms: i64,
) -> SystemStatusReadModel {
    let mut warnings = Vec::new();

    let (storage_mode, configured_db_primary) =
        match crate::workbench_sqlite_storage_mode::storage_mode_for(workflow_state_path) {
            crate::workbench_sqlite_storage_mode::StorageMode::JsonOnly { .. } => {
                ("json_only".to_string(), false)
            }
            crate::workbench_sqlite_storage_mode::StorageMode::DbPrimaryJsonProjection(_) => {
                ("db_primary".to_string(), true)
            }
        };

    let storage_healthy = if configured_db_primary {
        match crate::workbench_sqlite_storage_mode::db_primary_health_snapshot(workflow_state_path) {
            Some(Ok(())) => true,
            Some(Err(reason)) => {
                warnings.push(format!(
                    "DB 主写已冻结，本进程已降级 json_only（数据无损，需重 seed 恢复）：{reason}"
                ));
                false
            }
            None => {
                warnings.push(
                    "本进程还没跑启动对账，DB 主写健康判不了（保守按不健康报）。".to_string(),
                );
                false
            }
        }
    } else {
        true
    };

    let audit_events = match crate::read_workflow_state_value(workflow_state_path) {
        Ok(value) => value
            .get("audit_events")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_else(|| {
                warnings.push("主 store 没有 audit_events 数组，观察期与降级信息读不到。".to_string());
                Vec::new()
            }),
        Err(error) => {
            warnings.push(format!(
                "主 store 读不了，观察期与降级信息读不到：{error}"
            ));
            Vec::new()
        }
    };

    // 取**最新**一条 initialized＝「重开」口径（07-14 拍板：重 seed → 观察期重开；见文件头）。
    let observation_started_at_ms =
        latest_event(&audit_events, EVENT_STORAGE_MODE_INITIALIZED).and_then(event_at_ms);
    let observation_day = match observation_started_at_ms {
        Some(started_at_ms) => observation_day_from(started_at_ms, now_ms, &mut warnings),
        None => 0,
    };

    let last_degradation = latest_event(&audit_events, EVENT_STORAGE_MODE_DEGRADED).map(|event| {
        SystemStatusDegradation {
            at_ms: event_at_ms(event).unwrap_or(0),
            reason_human: event
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| "DB 主写曾降级 json_only（原因没留下人话）。".to_string()),
        }
    });

    let recent_catches = Vec::new();
    warnings.push(format!(
        "最近拦截暂为空：拦截账本是仓内人手维护的开发档案（docs/harness-catch-log.md），运行时没有数据源；要真填需另立写点（上限留 {RECENT_CATCH_LIMIT} 条）。"
    ));

    SystemStatusReadModel {
        storage_mode,
        storage_healthy,
        observation_day,
        last_degradation,
        recent_catches,
        gate_summary: Some(gate_summary()),
        warnings,
    }
}

/// 写解封闸一句话：由 `commands.rs` 编译期常量派生（常量改了这句自动跟着改·不会各说各话）。
fn gate_summary() -> String {
    format!(
        "{} 写解封·仅此项目（单一写根·其它真实项目仍封）；固定测试项目 {} 照旧可跑。",
        crate::STATION_4_WRITE_PROJECT_ROOT,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT
    )
}

/// 观察期第 N 天：切换当天 = 第 1 天。时钟回拨/审计时间在未来 → 保守报第 1 天 + warning。
fn observation_day_from(started_at_ms: i64, now_ms: i64, warnings: &mut Vec<String>) -> u32 {
    if now_ms < started_at_ms {
        warnings.push(
            "观察期起点比当前时间还晚（时钟回拨或审计时间异常），按第 1 天报。".to_string(),
        );
        return 1;
    }
    let days = (now_ms - started_at_ms) / DAY_MS;
    // +1 = 切换当天算第 1 天；i64→u32 走 try_into 兜底，绝不 as 截断成小数字。
    u32::try_from(days.saturating_add(1)).unwrap_or(u32::MAX)
}

fn events_of_type<'a>(
    audit_events: &'a [serde_json::Value],
    event_type: &str,
) -> impl Iterator<Item = &'a serde_json::Value> {
    let event_type = event_type.to_string();
    audit_events.iter().filter(move |event| {
        event.get("event_type").and_then(serde_json::Value::as_str) == Some(event_type.as_str())
    })
}

/// 最晚一条：按 `created_at` 取最大；时间读不出的条目不参与比较（但仍可能是唯一条 → 兜底取末条）。
fn latest_event<'a>(
    audit_events: &'a [serde_json::Value],
    event_type: &str,
) -> Option<&'a serde_json::Value> {
    let latest_by_time = events_of_type(audit_events, event_type)
        .filter(|event| event_at_ms(event).is_some())
        .max_by_key(|event| event_at_ms(event).unwrap_or(i64::MIN));
    latest_by_time.or_else(|| events_of_type(audit_events, event_type).last())
}

/// 存储模式审计的 `created_at` = `unix_timestamp_string()` 的毫秒串（`append_startup_mode_audit`:668）。
/// 老数据/异常形状兜底再看 `created_at_ms`；都读不出 → None（调用方保守处理）。
fn event_at_ms(event: &serde_json::Value) -> Option<i64> {
    if let Some(created_at) = event.get("created_at").and_then(serde_json::Value::as_str) {
        if let Ok(at_ms) = created_at.trim().parse::<i64>() {
            return Some(at_ms);
        }
    }
    event
        .get("created_at_ms")
        .and_then(serde_json::Value::as_i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TempRoot(PathBuf);

    impl Drop for TempRoot {
        fn drop(&mut self) {
            // 模式缓存按路径缓存·temp 根一次性——收尾清掉本路径的缓存条目，
            // 防串味（M5-C 前科：测试摸到 live 配置）。
            crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
                &self.0.join("workflow-state").join("workflow-state.v0.json"),
            );
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const DAY: i64 = 24 * 60 * 60 * 1000;
    const CUTOVER_MS: i64 = 1_700_000_000_000;

    /// 全新 temp 根 + 一份真 workflow state（走产品自己的 bootstrap·不手搓 JSON）。
    fn fixture(label: &str) -> (TempRoot, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "system-status-read-model-{label}-{}-{}",
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
                name: "system status fixture".to_string(),
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

    /// 直接把审计事件塞进主 store 的 audit_events（读模型的输入面·不经写点）。
    fn push_audit(state_path: &Path, event_type: &str, created_at_ms: i64, reason: &str) {
        let mut value = crate::read_workflow_state_value(state_path).expect("read state");
        value["audit_events"]
            .as_array_mut()
            .expect("audit_events array")
            .push(json!({
                "event_id": format!("{event_type}:{created_at_ms}"),
                "event_type": event_type,
                "target_ref": "db-hash",
                "actor_ref": "workbench_storage_mode",
                "created_at": created_at_ms.to_string(),
                "reason": reason,
            }));
        fs::write(
            state_path,
            serde_json::to_string_pretty(&value).expect("serialize state"),
        )
        .expect("write state");
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

    // 无 storage-mode 配置 → fail-closed 成 json_only；此时「配置如此」= 健康，观察期没开 = 第 0 天。
    #[test]
    fn json_only_without_storage_mode_config_is_healthy_and_outside_observation_period() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("json-only");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );

        let model = load_system_status_read_model_at(&state_path, CUTOVER_MS);

        assert_eq!(model.storage_mode, "json_only");
        assert!(model.storage_healthy, "json_only 是配置态·不该报不健康");
        assert_eq!(model.observation_day, 0, "没进过观察期 → 第 0 天");
        assert_eq!(model.last_degradation, None);
        assert!(model.recent_catches.is_empty());
    }

    // 观察期第 N 天：重开当天 = 第 1 天；跨 2 整天 = 第 3 天。
    // 「重开」口径（07-14 拍板：重 seed → 观察期重开）：起点取**最新**一条 initialized，
    // 不是最早那条（任务包原文的「首条」口径已按拍板改，见文件头）。
    #[test]
    fn observation_day_counts_from_latest_initialized_audit_with_reseed_day_as_day_one() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("observation-day");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );
        // 故意乱序 + 两条：起点必须按时间取**最新**那条（重 seed 那条），不是数组末条、更不是最早那条。
        push_audit(
            &state_path,
            EVENT_STORAGE_MODE_INITIALIZED,
            CUTOVER_MS + DAY,
            "重 seed 后再次初始化。",
        );
        push_audit(
            &state_path,
            EVENT_STORAGE_MODE_INITIALIZED,
            CUTOVER_MS,
            "已完成 DB 主写与 JSON 投影启动对账。",
        );

        let same_day = load_system_status_read_model_at(&state_path, CUTOVER_MS + DAY + 3_600_000);
        assert_eq!(same_day.observation_day, 1, "重开当天 = 第 1 天（按最早那条算会错成第 2 天）");

        let two_days_later = load_system_status_read_model_at(&state_path, CUTOVER_MS + DAY + 2 * DAY);
        assert_eq!(two_days_later.observation_day, 3, "重开后跨 2 整天 = 第 3 天");
    }

    // 降级：取**最晚**一条 degraded 事件，reason 原样上脸（后端已写成人话·读模型不再包一层）。
    #[test]
    fn last_degradation_takes_latest_event_and_keeps_backend_human_reason() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("degradation");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );
        push_audit(
            &state_path,
            EVENT_STORAGE_MODE_DEGRADED,
            CUTOVER_MS,
            "旧的一次降级。",
        );
        push_audit(
            &state_path,
            EVENT_STORAGE_MODE_DEGRADED,
            CUTOVER_MS + DAY,
            "DB 主写已冻结：投影失败；本进程已降级 json_only，数据无损。",
        );

        let model = load_system_status_read_model_at(&state_path, CUTOVER_MS + 2 * DAY);

        let degradation = model.last_degradation.expect("应报最近一次降级");
        assert_eq!(degradation.at_ms, CUTOVER_MS + DAY, "要最晚那条·不是数组末条");
        assert!(
            degradation.reason_human.contains("数据无损"),
            "后端人话应原样透出：{}",
            degradation.reason_human
        );
    }

    // 闸摘要：由编译期常量派生——常量改了这句跟着改，不会各说各话。
    #[test]
    fn gate_summary_is_derived_from_unseal_constants() {
        let summary = gate_summary();
        assert!(
            summary.contains(crate::STATION_4_WRITE_PROJECT_ROOT),
            "闸摘要要点名写解封项目：{summary}"
        );
        assert!(summary.contains("仅此项目"), "要说清只解封了一个项目：{summary}");
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
        push_audit(
            &state_path,
            EVENT_STORAGE_MODE_INITIALIZED,
            CUTOVER_MS,
            "已完成 DB 主写与 JSON 投影启动对账。",
        );
        let before = mtime(&state_path);

        let _ = load_system_status_read_model_at(&state_path, CUTOVER_MS + DAY);
        let _ = load_system_status_read_model_at(&state_path, CUTOVER_MS + DAY);

        assert_eq!(before, mtime(&state_path), "读模型纯只读·不许写 workflow state");
    }

    // 主 store 坏了：软着陆——出 warnings、不 panic、不 Err 断首页。
    #[test]
    fn broken_workflow_state_soft_lands_with_warning() {
        let _guard = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let (_root, state_path) = fixture("broken");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &state_path,
        );
        fs::write(&state_path, "{ not json").expect("corrupt state");

        let model = load_system_status_read_model_at(&state_path, CUTOVER_MS);

        assert_eq!(model.observation_day, 0);
        assert!(
            model.warnings.iter().any(|warning| warning.contains("主 store 读不了")),
            "坏 store 要有人话报备：{:?}",
            model.warnings
        );
    }
}
