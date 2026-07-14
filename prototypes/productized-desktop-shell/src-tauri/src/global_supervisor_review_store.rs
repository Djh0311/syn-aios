// B1·全局主管复核记录 store（sidecar `global-supervisor-reviews.v1.json`）。
//
// 任务包：tasks/2026-07-07-phase-b1-global-supervisor-review-on-reports-v1.md
// 决策正本：decisions/2026-07-07-phase-b-advisory-supervisor-and-secretary-v1.md
//
// 定位：复核 agent **唯一的写入面**——按 (workflow_id, chain_started_at) 存取一轮链的复核意见；
// 审计事件走 store 内嵌 audit_events（照 plan_authorization_store 先例）→ 复核全程
// **不写 workflow state 文件一个字节**（比包字面「新 store + 审计」更干净的取舍，回交已报备）。
//
// 家族先例（plan_authorization_store）：sidecar 定位走 utils::store_paths / 原子写（tmp+rename+sync）/
// 写前备份进 backups/ + prune / revision 递增。**损坏跳过**：load 损坏 → 空店 + warning（不 Err 断面板）；
// 写时 sidecar.exists() 会先把坏文件备份再覆盖——尸体保留可查、不静默丢。
//
// 安全属性：意见不是闸——本 store 只存意见与审计，不含任何被执行机器消费的字段。

use crate::utils::store_paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORE_SCHEMA_VERSION: &str = "global_supervisor_review_store.v1";
const SIDECAR_NAME: &str = "global-supervisor-reviews.v1.json";
const BACKUP_PREFIX: &str = "global-supervisor-reviews.v1.";
const MAX_BACKUPS: usize = 10;

#[path = "global_supervisor_review_store_db_primary.rs"]
pub(crate) mod db_primary;

/// 每任务点评（LM 输出投影·serde 全 default 软着陆）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalSupervisorTaskVerdict {
    #[serde(default)]
    pub(crate) title: String,
    /// "ok" | "issue"（未知值归一化为 "issue"·保守）。
    #[serde(default)]
    pub(crate) verdict: String,
    #[serde(default)]
    pub(crate) comment: String,
}

/// 一轮链的复核记录。status="ready"（意见在）| "unavailable"（复核没跑成·可重试）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalSupervisorReviewRecord {
    #[serde(default)]
    pub(crate) review_id: String,
    #[serde(default)]
    pub(crate) project_id: String,
    #[serde(default)]
    pub(crate) workflow_id: String,
    /// 幂等键半边：链记录 started_at（毫秒字符串·与盘上 workflow_chain_runs 同源）。
    #[serde(default)]
    pub(crate) chain_started_at: String,
    #[serde(default)]
    pub(crate) status: String,
    /// "pass" | "needs_rework" | "needs_human_check"（未知归一化为 needs_human_check·保守）。
    #[serde(default)]
    pub(crate) overall: String,
    #[serde(default)]
    pub(crate) summary: String,
    /// "none" | "replan" | "human_verify"（未知归一化为 none·不给错按钮）。
    #[serde(default)]
    pub(crate) suggested_action: String,
    #[serde(default)]
    pub(crate) human_note: String,
    #[serde(default)]
    pub(crate) tasks: Vec<GlobalSupervisorTaskVerdict>,
    /// status="unavailable" 时的人话原因（供给类/解析失败等）。
    #[serde(default)]
    pub(crate) unavailable_reason: Option<String>,
    /// §10-1 换脑可定位（零成本半边）：本次复核用的模型与档案版本。
    #[serde(default)]
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) profile_version: String,
    #[serde(default)]
    pub(crate) created_at_ms: i64,
    #[serde(default)]
    pub(crate) updated_at_ms: i64,
}

/// store 内嵌审计事件（照 plan_authorization_store 先例·不写 workflow state）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalSupervisorReviewAuditEvent {
    #[serde(default)]
    pub(crate) event_id: String,
    /// 固定 "global_supervisor_review_recorded"。
    #[serde(default)]
    pub(crate) event_type: String,
    #[serde(default)]
    pub(crate) workflow_id: String,
    #[serde(default)]
    pub(crate) chain_started_at: String,
    #[serde(default)]
    pub(crate) review_status: String,
    #[serde(default)]
    pub(crate) actor_ref: String,
    #[serde(default)]
    pub(crate) created_at_ms: i64,
}

// ===== B2·批前边界意见（加法扩展·独立集合·旧 reviews/audit_events 语义 0-diff） =====
//
// B1（结果复核·按 workflow_id+chain_started_at）与 B2（批前边界·按 proposal_id）是同一 store 的两半：
// 各存各的集合、各带各的内嵌审计。新字段全 `#[serde(default)]` → 旧 sidecar 缺字段照样反序列化
// （loader 容忍缺字段·schema_version 沿用 v1 不 bump=零 gate 收益、避免动版本常量波及）。

/// 一份方案的批前边界意见记录。status="ready"（意见在）| "unavailable"（没跑成·可重试）。
/// 幂等键 = proposal_id（一份方案一条·[重试] 才 force 重跑）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalSupervisorBoundaryReviewRecord {
    #[serde(default)]
    pub(crate) review_id: String,
    #[serde(default)]
    pub(crate) project_id: String,
    /// 幂等键：方案 id（与 proposal store 同源）。
    #[serde(default)]
    pub(crate) proposal_id: String,
    #[serde(default)]
    pub(crate) status: String,
    /// "looks_ok" | "mismatch" | "caution"（未知/审批腔归一化为 caution·保守）。
    #[serde(default)]
    pub(crate) verdict: String,
    /// 点破的短句（目标错配/越界苗头/验收缺/风险漏报）。
    #[serde(default)]
    pub(crate) points: Vec<String>,
    #[serde(default)]
    pub(crate) summary: String,
    /// status="unavailable" 时的人话原因（供给类/解析失败等）。
    #[serde(default)]
    pub(crate) unavailable_reason: Option<String>,
    /// §10-1 换脑可定位（零成本半边）：本次意见用的模型与档案版本。
    #[serde(default)]
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) profile_version: String,
    #[serde(default)]
    pub(crate) created_at_ms: i64,
    #[serde(default)]
    pub(crate) updated_at_ms: i64,
}

/// B2 内嵌审计事件（独立于 B1 的 audit_events·照同款内嵌先例）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalSupervisorBoundaryReviewAuditEvent {
    #[serde(default)]
    pub(crate) event_id: String,
    /// 固定 "global_supervisor_boundary_review_recorded"。
    #[serde(default)]
    pub(crate) event_type: String,
    #[serde(default)]
    pub(crate) proposal_id: String,
    #[serde(default)]
    pub(crate) review_status: String,
    #[serde(default)]
    pub(crate) actor_ref: String,
    #[serde(default)]
    pub(crate) created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalSupervisorReviewStoreV1 {
    pub(crate) schema_version: String,
    pub(crate) revision: i64,
    pub(crate) updated_at_ms: i64,
    #[serde(default)]
    pub(crate) reviews: Vec<GlobalSupervisorReviewRecord>,
    #[serde(default)]
    pub(crate) audit_events: Vec<GlobalSupervisorReviewAuditEvent>,
    /// B2·批前边界意见（加法·按 proposal_id）。旧 sidecar 缺此字段 → 空 vec。
    #[serde(default)]
    pub(crate) boundary_reviews: Vec<GlobalSupervisorBoundaryReviewRecord>,
    /// B2 内嵌审计（加法）。旧 sidecar 缺此字段 → 空 vec。
    #[serde(default)]
    pub(crate) boundary_audit_events: Vec<GlobalSupervisorBoundaryReviewAuditEvent>,
}

fn empty_store(timestamp_ms: i64) -> GlobalSupervisorReviewStoreV1 {
    GlobalSupervisorReviewStoreV1 {
        schema_version: STORE_SCHEMA_VERSION.to_string(),
        revision: 0,
        updated_at_ms: timestamp_ms,
        reviews: Vec::new(),
        audit_events: Vec::new(),
        boundary_reviews: Vec::new(),
        boundary_audit_events: Vec::new(),
    }
}

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    store_paths::sidecar_path(workflow_state_path, SIDECAR_NAME, "全局主管复核")
}

/// 读店（**损坏跳过**语义）：不存在 → 空店；损坏 → 空店 + 一条人话 warning（不 Err 断面板）。
/// 坏文件不在此处理——下次写盘时 `write_store_atomic` 的写前备份会把尸体收进 backups/。
pub(crate) fn load_store_soft(
    workflow_state_path: &Path,
    timestamp_ms: i64,
) -> (GlobalSupervisorReviewStoreV1, Vec<String>) {
    let sidecar = match sidecar_path(workflow_state_path) {
        Ok(path) => path,
        Err(error) => return (empty_store(timestamp_ms), vec![error]),
    };
    if !sidecar.exists() {
        return (empty_store(timestamp_ms), Vec::new());
    }
    let text = match fs::read_to_string(&sidecar) {
        Ok(text) => text,
        Err(error) => {
            return (
                empty_store(timestamp_ms),
                vec![format!(
                    "读取全局主管复核 sidecar 失败（按空店继续）{}：{error}",
                    sidecar.display()
                )],
            );
        }
    };
    match serde_json::from_str::<GlobalSupervisorReviewStoreV1>(&text) {
        Ok(store) => (store, Vec::new()),
        Err(error) => (
            empty_store(timestamp_ms),
            vec![format!(
                "全局主管复核 sidecar JSON 损坏（按空店继续·下次写盘会先备份坏文件）{}：{error}",
                sidecar.display()
            )],
        ),
    }
}

/// 按幂等键 (workflow_id, chain_started_at) 找一条复核记录。
pub(crate) fn find_review<'a>(
    store: &'a GlobalSupervisorReviewStoreV1,
    workflow_id: &str,
    chain_started_at: &str,
) -> Option<&'a GlobalSupervisorReviewRecord> {
    store.reviews.iter().find(|review| {
        review.workflow_id == workflow_id && review.chain_started_at == chain_started_at
    })
}

/// upsert 一条复核记录（同幂等键替换、否则追加）+ 内嵌审计 + revision 递增 + 原子写。
/// **append/upsert 只此一店**——复核不写任何其他 store。
pub(crate) fn upsert_review(
    workflow_state_path: &Path,
    record: GlobalSupervisorReviewRecord,
    actor_ref: &str,
    timestamp_ms: i64,
) -> Result<GlobalSupervisorReviewStoreV1, String> {
    if record.workflow_id.trim().is_empty() || record.chain_started_at.trim().is_empty() {
        return Err("复核记录缺 workflow_id / chain_started_at（幂等键），拒绝落库".to_string());
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    let (mut store, _warnings) = load_store_soft(workflow_state_path, timestamp_ms);
    let before_store = store.clone();
    let existing = store.reviews.iter().position(|review| {
        review.workflow_id == record.workflow_id
            && review.chain_started_at == record.chain_started_at
    });
    let mut record = record;
    record.updated_at_ms = timestamp_ms;
    match existing {
        Some(index) => {
            // 保留首次 created_at（重试/force 重跑是同一轮的更新，不伪造新生时刻）。
            let created = store.reviews[index].created_at_ms;
            record.created_at_ms = if created > 0 { created } else { timestamp_ms };
            store.reviews[index] = record.clone();
        }
        None => {
            record.created_at_ms = timestamp_ms;
            store.reviews.push(record.clone());
        }
    }
    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.audit_events.push(GlobalSupervisorReviewAuditEvent {
        event_id: format!(
            "global-supervisor-review:{}:{}:{timestamp_ms}",
            record.workflow_id, record.chain_started_at
        ),
        event_type: "global_supervisor_review_recorded".to_string(),
        workflow_id: record.workflow_id.clone(),
        chain_started_at: record.chain_started_at.clone(),
        review_status: record.status.clone(),
        actor_ref: actor_ref.to_string(),
        created_at_ms: timestamp_ms,
    });
    db_primary::write_store_with_db_primary(
        workflow_state_path,
        &sidecar,
        &before_store,
        &store,
        timestamp_ms,
    )?;
    Ok(store)
}

/// B2·按幂等键 proposal_id 找一条批前边界意见记录。
pub(crate) fn find_boundary_review<'a>(
    store: &'a GlobalSupervisorReviewStoreV1,
    proposal_id: &str,
) -> Option<&'a GlobalSupervisorBoundaryReviewRecord> {
    store
        .boundary_reviews
        .iter()
        .find(|review| review.proposal_id == proposal_id)
}

/// B2·upsert 一条批前边界意见（同 proposal_id 替换、否则追加）+ 内嵌 B2 审计 + revision 递增 + 原子写。
/// 复用 B1 同款原子写机制；**只碰 boundary_reviews / boundary_audit_events**——旧 reviews/audit_events 不动。
pub(crate) fn upsert_boundary_review(
    workflow_state_path: &Path,
    record: GlobalSupervisorBoundaryReviewRecord,
    actor_ref: &str,
    timestamp_ms: i64,
) -> Result<GlobalSupervisorReviewStoreV1, String> {
    if record.proposal_id.trim().is_empty() {
        return Err("批前边界意见记录缺 proposal_id（幂等键），拒绝落库".to_string());
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    let (mut store, _warnings) = load_store_soft(workflow_state_path, timestamp_ms);
    let before_store = store.clone();
    let existing = store
        .boundary_reviews
        .iter()
        .position(|review| review.proposal_id == record.proposal_id);
    let mut record = record;
    record.updated_at_ms = timestamp_ms;
    match existing {
        Some(index) => {
            // 保留首次 created_at（重试/force 重跑是同一份方案的更新，不伪造新生时刻）。
            let created = store.boundary_reviews[index].created_at_ms;
            record.created_at_ms = if created > 0 { created } else { timestamp_ms };
            store.boundary_reviews[index] = record.clone();
        }
        None => {
            record.created_at_ms = timestamp_ms;
            store.boundary_reviews.push(record.clone());
        }
    }
    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store
        .boundary_audit_events
        .push(GlobalSupervisorBoundaryReviewAuditEvent {
            event_id: format!(
                "global-supervisor-boundary-review:{}:{timestamp_ms}",
                record.proposal_id
            ),
            event_type: "global_supervisor_boundary_review_recorded".to_string(),
            proposal_id: record.proposal_id.clone(),
            review_status: record.status.clone(),
            actor_ref: actor_ref.to_string(),
            created_at_ms: timestamp_ms,
        });
    db_primary::write_store_with_db_primary(
        workflow_state_path,
        &sidecar,
        &before_store,
        &store,
        timestamp_ms,
    )?;
    Ok(store)
}

/// 原子写 + 写前备份 + prune（照 plan_authorization_store::write_store_atomic 先例）。
fn write_store_atomic(
    sidecar: &Path,
    store: &GlobalSupervisorReviewStoreV1,
    timestamp_ms: i64,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("全局主管复核 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!(
                "创建全局主管复核备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?;
        let backup = backup_dir.join(format!(
            "{BACKUP_PREFIX}{timestamp_ms}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup).map_err(|error| {
            format!(
                "备份全局主管复核 sidecar 失败 {}：{error}",
                backup.display()
            )
        })?;
        prune_backups(&backup_dir)?;
    }
    let temp_path = parent.join(format!(".{BACKUP_PREFIX}{timestamp_ms}.tmp"));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("全局主管复核 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!(
                "创建全局主管复核临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!(
                "写入全局主管复核临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "同步全局主管复核临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换全局主管复核 sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

/// 只留最近 MAX_BACKUPS 份备份（照 plan_authorization_store::prune_backups 先例）。
fn prune_backups(backup_dir: &Path) -> Result<(), String> {
    let mut backups = fs::read_dir(backup_dir)
        .map_err(|error| format!("读取备份目录失败 {}：{error}", backup_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with(BACKUP_PREFIX))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    backups.sort();
    while backups.len() > MAX_BACKUPS {
        let victim = backups.remove(0);
        fs::remove_file(&victim)
            .map_err(|error| format!("清理旧备份失败 {}：{error}", victim.display()))?;
    }
    Ok(())
}

// B3·整店只读 load 命令（照 load_formal_memory_store 家族先例·store 本体语义 0-diff）：
// 秘书「待你拍板」面要读主管两类意见。走 soft 语义（不存在/损坏 → 空店），秘书面板零炸优先；
// warnings 属诊断细节、此命令不透出（下次写盘自会备份坏文件）。只读，无任何写路径。
#[tauri::command]
pub(crate) fn load_global_supervisor_review_store(
    state: tauri::State<'_, crate::AppState>,
) -> Result<GlobalSupervisorReviewStoreV1, String> {
    let (store, _warnings) =
        load_store_soft(&state.workflow_state_path, crate::unix_timestamp_ms());
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_state_path(tag: &str) -> (PathBuf, PathBuf) {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("gsr-store-{tag}-{uniq}"));
        fs::create_dir_all(&dir).expect("tmp dir");
        (dir.clone(), dir.join("workflow-state.v0.json"))
    }

    fn record(
        workflow_id: &str,
        chain_started_at: &str,
        status: &str,
    ) -> GlobalSupervisorReviewRecord {
        GlobalSupervisorReviewRecord {
            review_id: format!("review:{workflow_id}:{chain_started_at}"),
            project_id: "proj".to_string(),
            workflow_id: workflow_id.to_string(),
            chain_started_at: chain_started_at.to_string(),
            status: status.to_string(),
            overall: "pass".to_string(),
            summary: "都做完了".to_string(),
            suggested_action: "none".to_string(),
            model: "codex-cli-default".to_string(),
            profile_version: "global-supervisor-profile.v1".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn roundtrip_upsert_find_and_revision_bump() {
        let (dir, state_path) = tmp_state_path("roundtrip");
        let store = upsert_review(
            &state_path,
            record("wf-1", "1000", "ready"),
            "tester",
            1_000,
        )
        .expect("upsert");
        assert_eq!(store.revision, 1);
        let (loaded, warnings) = load_store_soft(&state_path, 2_000);
        assert!(warnings.is_empty(), "干净店无 warning：{warnings:?}");
        let found = find_review(&loaded, "wf-1", "1000").expect("应找到");
        assert_eq!(found.status, "ready");
        assert_eq!(found.overall, "pass");
        assert_eq!(found.model, "codex-cli-default", "§10-1 model 字段落盘");
        assert_eq!(found.profile_version, "global-supervisor-profile.v1");
        // 审计内嵌本店（不写 workflow state）。
        assert_eq!(loaded.audit_events.len(), 1);
        assert_eq!(
            loaded.audit_events[0].event_type,
            "global_supervisor_review_recorded"
        );
        // 同键 upsert 替换不追加 + revision 递增 + created_at 保留。
        let created_first = found.created_at_ms;
        let store2 = upsert_review(
            &state_path,
            record("wf-1", "1000", "ready"),
            "tester",
            3_000,
        )
        .expect("upsert2");
        assert_eq!(store2.revision, 2);
        assert_eq!(store2.reviews.len(), 1, "同幂等键替换不追加");
        assert_eq!(
            store2.reviews[0].created_at_ms, created_first,
            "created_at 保留首次"
        );
        assert_eq!(store2.reviews[0].updated_at_ms, 3_000);
        // 不同键追加。
        let store3 = upsert_review(
            &state_path,
            record("wf-1", "2000", "unavailable"),
            "tester",
            4_000,
        )
        .expect("upsert3");
        assert_eq!(store3.reviews.len(), 2);
        assert!(find_review(&store3, "wf-1", "2000").is_some());
        assert!(find_review(&store3, "wf-1", "9999").is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupted_sidecar_soft_lands_and_backup_preserves_corpse() {
        let (dir, state_path) = tmp_state_path("corrupt");
        let sidecar = sidecar_path(&state_path).expect("sidecar path");
        fs::write(&sidecar, "{ 这不是合法 json").expect("write corrupt");
        // 损坏跳过：空店 + warning，不 Err。
        let (loaded, warnings) = load_store_soft(&state_path, 1_000);
        assert!(loaded.reviews.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].contains("损坏"),
            "warning 人话：{}",
            warnings[0]
        );
        // 写盘：坏文件先进 backups/（尸体保留可查），再原子写新店。
        upsert_review(
            &state_path,
            record("wf-1", "1000", "ready"),
            "tester",
            2_000,
        )
        .expect("upsert over corrupt");
        let backup_dir = dir.join("backups");
        let corpse_kept = fs::read_dir(&backup_dir)
            .expect("backups dir")
            .filter_map(|e| e.ok())
            .any(|e| {
                fs::read_to_string(e.path())
                    .map(|t| t.contains("这不是合法 json"))
                    .unwrap_or(false)
            });
        assert!(corpse_kept, "坏文件应备份保留");
        let (reloaded, rewarnings) = load_store_soft(&state_path, 3_000);
        assert!(rewarnings.is_empty(), "新店应干净");
        assert!(find_review(&reloaded, "wf-1", "1000").is_some());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_idempotency_key_rejected() {
        let (dir, state_path) = tmp_state_path("nokey");
        let mut bad = record("", "1000", "ready");
        bad.workflow_id = String::new();
        assert!(upsert_review(&state_path, bad, "tester", 1_000).is_err());
        let mut bad2 = record("wf-1", "", "ready");
        bad2.chain_started_at = String::new();
        assert!(upsert_review(&state_path, bad2, "tester", 1_000).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    // ===== B2·批前边界意见（加法自证：旧 reviews 集合语义 0-diff） =====

    fn boundary_record(proposal_id: &str, status: &str) -> GlobalSupervisorBoundaryReviewRecord {
        GlobalSupervisorBoundaryReviewRecord {
            review_id: format!("boundary:{proposal_id}"),
            project_id: "proj".to_string(),
            proposal_id: proposal_id.to_string(),
            status: status.to_string(),
            verdict: "mismatch".to_string(),
            points: vec!["你要动手，这方案不改任何文件".to_string()],
            summary: "目标与方案对不上".to_string(),
            model: "codex-cli-default".to_string(),
            profile_version: "global-supervisor-boundary-profile.v1".to_string(),
            ..Default::default()
        }
    }

    // §4：boundary_reviews 往返（upsert/find/同键替换/revision）+ 内嵌 B2 审计 + 旧 reviews 集合不受扰。
    #[test]
    fn boundary_roundtrip_and_leaves_b1_reviews_untouched() {
        let (dir, state_path) = tmp_state_path("boundary");
        // 先放一条 B1 结果复核，再放 B2 边界意见——两半各存各的、互不干扰。
        upsert_review(
            &state_path,
            record("wf-1", "1000", "ready"),
            "tester",
            1_000,
        )
        .expect("b1");
        let store = upsert_boundary_review(
            &state_path,
            boundary_record("prop-1", "ready"),
            "global_supervisor_agent",
            2_000,
        )
        .expect("b2 upsert");
        assert_eq!(store.revision, 2, "两次写各递增一次");
        // 加法自证：B1 的 reviews / audit_events 一条不少、语义不变。
        assert_eq!(store.reviews.len(), 1, "旧 reviews 集合不受扰");
        assert_eq!(store.audit_events.len(), 1, "旧 audit_events 集合不受扰");
        assert!(find_review(&store, "wf-1", "1000").is_some(), "B1 记录仍在");
        // B2 落 boundary_reviews + boundary_audit_events。
        let (loaded, warnings) = load_store_soft(&state_path, 3_000);
        assert!(warnings.is_empty(), "干净店无 warning：{warnings:?}");
        let found = find_boundary_review(&loaded, "prop-1").expect("应找到边界意见");
        assert_eq!(found.verdict, "mismatch");
        assert_eq!(found.points.len(), 1);
        assert_eq!(found.model, "codex-cli-default", "§10-1 model 落盘");
        assert_eq!(loaded.boundary_audit_events.len(), 1);
        assert_eq!(
            loaded.boundary_audit_events[0].event_type,
            "global_supervisor_boundary_review_recorded"
        );
        assert_eq!(loaded.boundary_audit_events[0].proposal_id, "prop-1");
        // 同 proposal_id upsert 替换不追加 + created_at 保留首次。
        let created_first = found.created_at_ms;
        let store2 = upsert_boundary_review(
            &state_path,
            boundary_record("prop-1", "ready"),
            "global_supervisor_agent",
            4_000,
        )
        .expect("b2 upsert2");
        assert_eq!(
            store2.boundary_reviews.len(),
            1,
            "同 proposal_id 替换不追加"
        );
        assert_eq!(
            store2.boundary_reviews[0].created_at_ms, created_first,
            "created_at 保留首次"
        );
        assert_eq!(store2.boundary_reviews[0].updated_at_ms, 4_000);
        // 不同 proposal_id 追加。
        let store3 = upsert_boundary_review(
            &state_path,
            boundary_record("prop-2", "unavailable"),
            "global_supervisor_agent",
            5_000,
        )
        .expect("b2 upsert3");
        assert_eq!(store3.boundary_reviews.len(), 2);
        assert!(find_boundary_review(&store3, "prop-2").is_some());
        assert!(find_boundary_review(&store3, "prop-x").is_none());
        // B1 仍 1 条（B2 追加没碰旧集合）。
        assert_eq!(store3.reviews.len(), 1, "写三次 B2 后 B1 仍一条");
        let _ = fs::remove_dir_all(dir);
    }

    // §4：旧格式 sidecar（无 boundary_* 字段）loader 容忍 → 空 vec 反序列化，不 Err。
    #[test]
    fn legacy_sidecar_without_boundary_fields_loads_soft() {
        let (dir, state_path) = tmp_state_path("legacy");
        let sidecar = sidecar_path(&state_path).expect("sidecar path");
        // 旧 v1 店：只有 reviews/audit_events，没有 boundary_* 字段。
        fs::write(
            &sidecar,
            r#"{"schema_version":"global_supervisor_review_store.v1","revision":1,"updated_at_ms":10,"reviews":[],"audit_events":[]}"#,
        )
        .expect("write legacy");
        let (loaded, warnings) = load_store_soft(&state_path, 1_000);
        assert!(warnings.is_empty(), "旧格式不该报 warning：{warnings:?}");
        assert!(loaded.boundary_reviews.is_empty(), "缺字段 → 空 vec");
        assert!(loaded.boundary_audit_events.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn boundary_missing_proposal_id_rejected() {
        let (dir, state_path) = tmp_state_path("bnokey");
        let bad = boundary_record("", "ready");
        assert!(upsert_boundary_review(&state_path, bad, "tester", 1_000).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
