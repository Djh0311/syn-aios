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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GlobalSupervisorReviewStoreV1 {
    pub(crate) schema_version: String,
    pub(crate) revision: i64,
    pub(crate) updated_at_ms: i64,
    #[serde(default)]
    pub(crate) reviews: Vec<GlobalSupervisorReviewRecord>,
    #[serde(default)]
    pub(crate) audit_events: Vec<GlobalSupervisorReviewAuditEvent>,
}

fn empty_store(timestamp_ms: i64) -> GlobalSupervisorReviewStoreV1 {
    GlobalSupervisorReviewStoreV1 {
        schema_version: STORE_SCHEMA_VERSION.to_string(),
        revision: 0,
        updated_at_ms: timestamp_ms,
        reviews: Vec::new(),
        audit_events: Vec::new(),
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
    write_store_atomic(&sidecar, &store, timestamp_ms)?;
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
}
