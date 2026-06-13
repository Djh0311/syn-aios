use crate::utils::hash::short_hash;
use crate::{
    FormalMemoryStoreV1, MemoryCandidateStoreV1, MemoryLintFindingSeverity,
    MemoryLintFindingStatus, MemoryLintRunInput, MemoryLintRunIntent, MemoryLintRunOutput,
    MemoryLintRunRecord, MemoryLintRunStatus, MemoryLintStoreSummary, MemoryLintStoreV1,
};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORE_VERSION: &str = "memory_lint_store.v1";
const SIDECAR_NAME: &str = "memory-lint.v1.json";
const LOCK_NAME: &str = ".memory-lint.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导 memory lint sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<MemoryLintStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp));
    }
    let text = fs::read_to_string(&sidecar).map_err(|error| {
        format!(
            "读取 memory lint sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    let store: MemoryLintStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "memory lint sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn run_lint(
    workflow_state_path: &Path,
    input: &MemoryLintRunInput,
    timestamp: &str,
    write_id: &str,
) -> Result<MemoryLintRunOutput, String> {
    crate::control_core::validate_memory_lint_run(
        &input.project_root,
        &input.actor_id,
        &input.actor_role,
        memory_lint_intent_name(input.lint_intent),
    )?;
    let formal_store = crate::formal_memory_store::load_store(workflow_state_path, timestamp)?;
    let candidate_store =
        crate::memory_candidate_store::load_store(workflow_state_path, timestamp)?;
    let observation_store = crate::observation_store::load_store(workflow_state_path, timestamp)?;
    let entity_relation_store =
        crate::memory_entity_relation_store::load_store(workflow_state_path, timestamp)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("memory lint sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 memory lint sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    validate_expected_revisions(input, &formal_store, &candidate_store, &store)?;

    let generated = crate::memory_lint_engine::build_findings(
        input,
        &formal_store,
        &candidate_store,
        &observation_store,
        &entity_relation_store,
        timestamp,
    )?;
    let finding_ids = generated
        .iter()
        .map(|finding| finding.finding_id.clone())
        .collect::<BTreeSet<_>>();
    let blocking_count = generated
        .iter()
        .filter(|finding| crate::memory_lint_engine::is_open_blocking(finding))
        .count();
    let existing_ids = store
        .findings
        .iter()
        .map(|finding| finding.finding_id.clone())
        .collect::<BTreeSet<_>>();
    let new_findings = generated
        .into_iter()
        .filter(|finding| !existing_ids.contains(&finding.finding_id))
        .collect::<Vec<_>>();
    let run_id = format!(
        "memlint-run:v1:{}:{}",
        timestamp,
        short_hash(&format!(
            "{}:{}:{}",
            memory_lint_intent_name(input.lint_intent),
            input.candidate_key.clone().unwrap_or_default(),
            input.task_id.clone().unwrap_or_default()
        ))
    );
    let report = if is_maintenance_intent(input.lint_intent) {
        Some(crate::memory_lint_engine::build_maintenance_report(
            input,
            &formal_store,
            &candidate_store,
            &observation_store,
            &entity_relation_store,
            store.revision,
            &store.findings,
            &new_findings,
            &run_id,
            timestamp,
        )?)
    } else {
        None
    };
    let run = MemoryLintRunRecord {
        run_id,
        lint_intent: input.lint_intent,
        actor_id: input.actor_id.clone(),
        actor_role: input.actor_role.clone(),
        finding_ids: finding_ids.into_iter().collect(),
        blocking_count,
        status: if blocking_count > 0 {
            MemoryLintRunStatus::Blocked
        } else {
            MemoryLintRunStatus::Succeeded
        },
        reason: run_reason(input, blocking_count),
        report_id: report.as_ref().map(|report| report.report_id.clone()),
        created_at: timestamp.to_string(),
    };

    if input.dry_run.unwrap_or(false) {
        drop(lock);
        return Ok(output(store, run, report, new_findings, blocking_count));
    }

    store.project_id = input.project_id.clone().or(store.project_id);
    store.workflow_id = input.workflow_id.clone().or(store.workflow_id);
    store.findings.extend(new_findings.clone());
    store.runs.push(run.clone());
    if let Some(report) = report.clone() {
        store.maintenance_reports.push(report);
    }
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);

    Ok(output(store, run, report, new_findings, blocking_count))
}

pub(crate) fn summarize_store(store: &MemoryLintStoreV1) -> MemoryLintStoreSummary {
    let open_count = store
        .findings
        .iter()
        .filter(|finding| finding.status == MemoryLintFindingStatus::Open)
        .count();
    let blocking_count = store
        .findings
        .iter()
        .filter(|finding| {
            finding.status == MemoryLintFindingStatus::Open
                && finding.severity == MemoryLintFindingSeverity::Blocking
        })
        .count();
    let needs_review_count = store
        .findings
        .iter()
        .filter(|finding| {
            finding.status == MemoryLintFindingStatus::Open
                && finding.severity == MemoryLintFindingSeverity::NeedsReview
        })
        .count();
    let info_count = store
        .findings
        .iter()
        .filter(|finding| {
            finding.status == MemoryLintFindingStatus::Open
                && finding.severity == MemoryLintFindingSeverity::Info
        })
        .count();
    MemoryLintStoreSummary {
        sidecar_name: SIDECAR_NAME.to_string(),
        revision: store.revision,
        finding_count: store.findings.len(),
        open_count,
        blocking_count,
        needs_review_count,
        info_count,
        recent_run: store.runs.last().cloned(),
        recent_maintenance_report: store.maintenance_reports.last().cloned(),
        display_text: format!(
            "记忆 lint / maintenance 摘要：open {open_count} / blocking {blocking_count} / needs_review {needs_review_count} / info {info_count}；维护任务只生成 finding；不会自动修改正式记忆"
        ),
        warnings: store.warnings.clone(),
    }
}

fn output(
    store: MemoryLintStoreV1,
    run: MemoryLintRunRecord,
    report: Option<crate::MemoryMaintenanceReport>,
    new_findings: Vec<crate::MemoryLintFinding>,
    blocking_count: usize,
) -> MemoryLintRunOutput {
    let open_count = store
        .findings
        .iter()
        .chain(new_findings.iter())
        .filter(|finding| finding.status == MemoryLintFindingStatus::Open)
        .count();
    MemoryLintRunOutput {
        store,
        run,
        report,
        new_findings,
        blocking_count,
        open_count,
        warnings: vec![
            "memory_lint_findings_only_no_formal_memory_mutation".to_string(),
            "blocking_finding_blocks_adoption_or_task_packet".to_string(),
        ],
    }
}

fn validate_expected_revisions(
    input: &MemoryLintRunInput,
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    lint_store: &MemoryLintStoreV1,
) -> Result<(), String> {
    if let Some(expected) = input.expected_formal_store_revision {
        if expected != formal_store.revision {
            return Err(format!(
                "memory_lint_formal_store_conflict: expected revision {expected}, actual {}",
                formal_store.revision
            ));
        }
    }
    if let Some(expected) = input.expected_candidate_store_revision {
        if expected != candidate_store.revision {
            return Err(format!(
                "memory_lint_candidate_store_conflict: expected revision {expected}, actual {}",
                candidate_store.revision
            ));
        }
    }
    if let Some(expected) = input.expected_lint_store_revision {
        if expected != lint_store.revision {
            return Err(format!(
                "memory_lint_store_conflict: expected revision {expected}, actual {}",
                lint_store.revision
            ));
        }
    }
    Ok(())
}

fn run_reason(input: &MemoryLintRunInput, blocking_count: usize) -> String {
    if blocking_count > 0 {
        return format!(
            "{} found {blocking_count} blocking finding(s)",
            memory_lint_intent_name(input.lint_intent)
        );
    }
    format!(
        "{} completed without blocking finding",
        memory_lint_intent_name(input.lint_intent)
    )
}

fn empty_store(timestamp: &str) -> MemoryLintStoreV1 {
    MemoryLintStoreV1 {
        store_version: STORE_VERSION.to_string(),
        project_id: None,
        workflow_id: None,
        revision: 0,
        findings: vec![],
        runs: vec![],
        maintenance_reports: vec![],
        updated_at: timestamp.to_string(),
        warnings: vec![
            "memory_lint_store_findings_only".to_string(),
            "memory_maintenance_reports_do_not_mutate_formal_memory".to_string(),
        ],
    }
}

fn validate_store(store: &MemoryLintStoreV1) -> Result<(), String> {
    if store.store_version != STORE_VERSION {
        return Err(format!(
            "memory lint store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.revision < 0 {
        return Err("memory lint revision 不能小于 0".to_string());
    }
    Ok(())
}

fn write_store_atomic(
    sidecar: &Path,
    store: &MemoryLintStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("memory lint sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!(
                "创建 memory lint 备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?;
        let backup = backup_dir.join(format!(
            "memory-lint.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup).map_err(|error| {
            format!(
                "备份 memory lint sidecar 失败 {}：{error}",
                backup.display()
            )
        })?;
        prune_backups(&backup_dir, "memory-lint.v1.")?;
    }
    let temp_path = parent.join(format!(".memory-lint.v1.{timestamp}.{write_id}.tmp"));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("memory lint sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!(
                "创建 memory lint 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!(
                "写入 memory lint 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "同步 memory lint 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换 memory lint sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn prune_backups(backup_dir: &Path, prefix: &str) -> Result<(), String> {
    let mut backups = fs::read_dir(backup_dir)
        .map_err(|error| {
            format!(
                "读取 memory lint 备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
        .collect::<Vec<_>>();
    backups.sort_by_key(|entry| entry.file_name());
    let remove_count = backups.len().saturating_sub(20);
    for entry in backups.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

fn memory_lint_intent_name(intent: MemoryLintRunIntent) -> &'static str {
    match intent {
        MemoryLintRunIntent::CandidateAdoptionGuard => "candidate_adoption_guard",
        MemoryLintRunIntent::TaskPacketGuard => "task_packet_guard",
        MemoryLintRunIntent::MaintenancePreview => "maintenance_preview",
        MemoryLintRunIntent::MaintenanceRun => "maintenance_run",
    }
}

fn is_maintenance_intent(intent: MemoryLintRunIntent) -> bool {
    matches!(
        intent,
        MemoryLintRunIntent::MaintenancePreview | MemoryLintRunIntent::MaintenanceRun
    )
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => {
                file.write_all(write_id.as_bytes()).map_err(|error| {
                    format!("写入 memory lint lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(format!("memory_lint_store_locked: {}", path.display()))
            }
            Err(error) => Err(format!(
                "创建 memory lint lock 失败 {}：{error}",
                path.display()
            )),
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
