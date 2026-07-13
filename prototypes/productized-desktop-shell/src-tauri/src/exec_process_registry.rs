use crate::utils::store_paths;
use crate::CodexLocalExecutionRequest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: &str = "exec_process_registry.v1";
const SIDECAR_NAME: &str = "exec-process-registry.v1.json";
const LOCK_NAME: &str = ".exec-process-registry.v1.lock";
const REAP_WAIT_ATTEMPTS: usize = 20;
const REAP_WAIT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct RegisteredProcess {
    pid: u32,
    run_id: String,
    started_at: String,
    cmdline_summary: String,
    #[serde(default)]
    process_group: bool,
    #[serde(default)]
    process_group_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct RegistryAuditEvent {
    event_id: String,
    event_type: String,
    created_at: String,
    reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ExecProcessRegistryStore {
    schema_version: String,
    revision: i64,
    entries: Vec<RegisteredProcess>,
    audit_events: Vec<RegistryAuditEvent>,
    warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedProcess {
    started_at: String,
    cmdline: String,
    process_group_id: Option<u32>,
}

trait ProcessOperations {
    fn inspect(&self, pid: u32) -> Result<Option<ObservedProcess>, String>;
    fn kill_and_wait(&self, pid: u32, process_group: bool) -> Result<(), String>;
}

struct SystemProcessOperations;

impl ProcessOperations for SystemProcessOperations {
    fn inspect(&self, pid: u32) -> Result<Option<ObservedProcess>, String> {
        let started_at = ps_field(pid, "lstart")?;
        let Some(started_at) = started_at else {
            return Ok(None);
        };
        let cmdline = ps_field(pid, "command")?
            .ok_or_else(|| format!("读取 PID {pid} 命令行失败：进程在两次核验之间退出"))?;
        let process_group_id =
            ps_field(pid, "pgid")?.and_then(|value| value.trim().parse::<u32>().ok());
        Ok(Some(ObservedProcess {
            started_at,
            cmdline,
            process_group_id,
        }))
    }

    fn kill_and_wait(&self, pid: u32, process_group: bool) -> Result<(), String> {
        let target = if process_group {
            format!("-{pid}")
        } else {
            pid.to_string()
        };
        let status = Command::new("/bin/kill")
            .arg("-KILL")
            .arg(&target)
            .status()
            .map_err(|error| format!("回收执行目标 {target} 时无法发送终止信号：{error}"))?;
        if !status.success() {
            return Err(format!("回收执行目标 {target} 时终止信号返回 {status}"));
        }
        for _ in 0..REAP_WAIT_ATTEMPTS {
            if self.inspect(pid)?.is_none() {
                return Ok(());
            }
            thread::sleep(REAP_WAIT_INTERVAL);
        }
        Err(format!("回收 PID {pid} 后仍未退出"))
    }
}

/// runner 成功 spawn 后的登记护栏。正常路径显式注销；提前返回时 Drop 同样清理登记。
pub(crate) struct ProcessRegistration {
    workflow_state_path: PathBuf,
    entry: Option<RegisteredProcess>,
}

/// manual relay 的真进程登记必须显式注销；异常路径 drop 不会清登记，确保下次启动仍能补收。
pub(crate) struct DurableProcessRegistration {
    workflow_state_path: PathBuf,
    entry: Option<RegisteredProcess>,
}

impl DurableProcessRegistration {
    pub(crate) fn unregister(mut self) -> Result<(), String> {
        let Some(entry) = self.entry.take() else {
            return Ok(());
        };
        unregister_entry(&self.workflow_state_path, &entry)
    }
}

impl ProcessRegistration {
    pub(crate) fn unregister(mut self) {
        self.unregister_inner();
    }

    fn unregister_inner(&mut self) {
        let Some(entry) = self.entry.take() else {
            return;
        };
        let _ = unregister_entry(&self.workflow_state_path, &entry);
    }
}

impl Drop for ProcessRegistration {
    fn drop(&mut self) {
        self.unregister_inner();
    }
}

/// 主管编排会话和 runner 子进程共用同一份登记/回收 sidecar；run-id 保持主管侧原值贯通。
pub(crate) fn register_supervisor_spawned_process(
    workflow_state_path: &Path,
    run_id: &str,
    pid: u32,
) -> ProcessRegistration {
    register_spawned_process_for(
        workflow_state_path,
        run_id,
        pid,
        false,
        &SystemProcessOperations,
    )
}

/// manual relay 把子进程设为独立进程组组长；登记时保留这个事实，异常退出后的启动回收才能
/// 同时终止外层 codex 包装进程与它派生的子进程。其它 runner 仍按单 PID 处理。
pub(crate) fn register_manual_relay_process_group(
    workflow_state_path: &Path,
    run_id: &str,
    pid: u32,
) -> Result<DurableProcessRegistration, String> {
    register_codex_process_group_with(
        workflow_state_path,
        run_id,
        pid,
        "manual relay",
        &SystemProcessOperations,
    )
}

/// local runner 与 manual relay 共用严格进程组登记：只有刚 spawn、身份可核验且
/// `pgid == pid` 的 codex exec 才能进入 sidecar。登记失败时调用方必须在发 prompt 前止损。
pub(crate) fn register_spawned_codex_process_group(
    request: &CodexLocalExecutionRequest,
    pid: u32,
) -> Result<DurableProcessRegistration, String> {
    let workflow_state_path = crate::default_workflow_state_path();
    let run_id = codex_local_process_run_id(request);
    #[cfg(test)]
    {
        let _ = (run_id, pid);
        return Ok(DurableProcessRegistration {
            workflow_state_path,
            entry: None,
        });
    }
    #[cfg(not(test))]
    register_codex_process_group_with(
        &workflow_state_path,
        &run_id,
        pid,
        "codex local runner",
        &SystemProcessOperations,
    )
}

fn codex_local_process_run_id(request: &CodexLocalExecutionRequest) -> String {
    durable_process_run_id(
        &request.operation_id,
        &[
            request.prompt_ref.as_str(),
            request.work_item_id.as_deref().unwrap_or_default(),
            request.session_id.as_deref().unwrap_or_default(),
            request.project_id.as_str(),
            request.workflow_id.as_str(),
            request.node_id.as_str(),
            request.target_cwd.as_str(),
        ],
    )
}

fn durable_process_run_id(operation: &str, identity_parts: &[&str]) -> String {
    let operation = operation.trim();
    let operation = if operation.is_empty() {
        "unknown"
    } else {
        operation
    };
    format!(
        "codex-local:{operation}:{}",
        crate::utils::hash::sha256_hex(&identity_parts.join("\0"))
    )
}

fn register_codex_process_group_with(
    workflow_state_path: &Path,
    run_id: &str,
    pid: u32,
    owner: &str,
    operations: &dyn ProcessOperations,
) -> Result<DurableProcessRegistration, String> {
    let observed = operations
        .inspect(pid)?
        .ok_or_else(|| format!("{owner} PID {pid} 在登记前已退出"))?;
    if !is_workbench_codex_exec(&observed.cmdline) {
        return Err(format!("{owner} PID {pid} 不是可识别的 codex exec"));
    }
    if observed.process_group_id != Some(pid) {
        return Err(format!(
            "{owner} PID {pid} 不是已核验的进程组组长（pgid={:?}）",
            observed.process_group_id
        ));
    }
    let entry = RegisteredProcess {
        pid,
        run_id: run_id.to_string(),
        started_at: observed.started_at,
        cmdline_summary: observed.cmdline,
        process_group: true,
        process_group_id: Some(pid),
    };
    register_entry(workflow_state_path, entry.clone())?;
    Ok(DurableProcessRegistration {
        workflow_state_path: workflow_state_path.to_path_buf(),
        entry: Some(entry),
    })
}

fn register_spawned_process_for(
    workflow_state_path: &Path,
    run_id: &str,
    pid: u32,
    process_group: bool,
    operations: &dyn ProcessOperations,
) -> ProcessRegistration {
    let workflow_state_path = workflow_state_path.to_path_buf();
    let observed = match operations.inspect(pid) {
        Ok(Some(observed)) if is_workbench_codex_exec(&observed.cmdline) => observed,
        _ => {
            return ProcessRegistration {
                workflow_state_path,
                entry: None,
            };
        }
    };
    let entry = RegisteredProcess {
        pid,
        run_id: run_id.to_string(),
        started_at: observed.started_at,
        cmdline_summary: observed.cmdline,
        process_group,
        process_group_id: process_group.then_some(pid),
    };
    let registered = register_entry(&workflow_state_path, entry.clone()).is_ok();
    ProcessRegistration {
        workflow_state_path,
        entry: registered.then_some(entry),
    }
}

/// App 启动时只检查本 sidecar 已登记的进程。登记外 PID 不枚举、不匹配、更不会终止。
pub(crate) fn reap_registered_orphans(workflow_state_path: &Path) -> Result<usize, String> {
    reap_registered_orphans_with(workflow_state_path, &SystemProcessOperations)
}

fn reap_registered_orphans_with(
    workflow_state_path: &Path,
    operations: &dyn ProcessOperations,
) -> Result<usize, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(0);
    }
    with_store(workflow_state_path, "startup-reap", |store| {
        let mut retained = Vec::new();
        let mut reclaimed = 0;
        for entry in store.entries.clone() {
            match operations.inspect(entry.pid) {
                Ok(None) => {
                    store.audit_events.push(registry_audit(
                        "exec_process_registry_entry_cleared",
                        &entry,
                        "登记的执行进程已自然退出，已清理遗留登记。",
                    ));
                }
                Ok(Some(observed)) if !same_process(&entry, &observed) => {
                    store.audit_events.push(registry_audit(
                        "exec_process_registry_identity_mismatch",
                        &entry,
                        "登记 PID 的启动时间或命令行已不符，已只注销登记，未终止任何进程。",
                    ));
                }
                Ok(Some(_)) => match operations.kill_and_wait(entry.pid, entry.process_group) {
                    Ok(()) => {
                        reclaimed += 1;
                        store.audit_events.push(registry_audit(
                            "exec_process_registry_orphan_reaped",
                            &entry,
                            &format!("回收了上次遗留的执行进程 · 启动于 {}。", entry.started_at),
                        ));
                    }
                    Err(error) => {
                        retained.push(entry.clone());
                        store.warnings.push(format!(
                            "登记执行进程 PID {} 回收失败，保留到下次启动重试：{error}",
                            entry.pid
                        ));
                    }
                },
                Err(error) => {
                    retained.push(entry.clone());
                    store.warnings.push(format!(
                        "无法核验登记执行进程 PID {}，未终止并保留到下次启动：{error}",
                        entry.pid
                    ));
                }
            }
        }
        store.entries = retained;
        Ok(reclaimed)
    })
}

fn register_entry(workflow_state_path: &Path, entry: RegisteredProcess) -> Result<(), String> {
    with_store(
        workflow_state_path,
        &format!("register-{}", entry.pid),
        |store| {
            store.entries.retain(|current| current.pid != entry.pid);
            store.entries.push(entry);
            Ok(())
        },
    )
}

fn unregister_entry(workflow_state_path: &Path, entry: &RegisteredProcess) -> Result<(), String> {
    with_store(
        workflow_state_path,
        &format!("unregister-{}", entry.pid),
        |store| {
            store.entries.retain(|current| current != entry);
            Ok(())
        },
    )
}

fn with_store<T>(
    workflow_state_path: &Path,
    write_id: &str,
    mutate: impl FnOnce(&mut ExecProcessRegistryStore) -> Result<T, String>,
) -> Result<T, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("执行进程登记 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建执行进程登记目录失败 {}：{error}", parent.display()))?;
    let lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)?;
    let mut store = load_store(&sidecar)?;
    let result = mutate(&mut store)?;
    store.revision += 1;
    write_store_atomic(&sidecar, &store)?;
    drop(lock);
    Ok(result)
}

fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    store_paths::sidecar_path(workflow_state_path, SIDECAR_NAME, "执行进程登记")
}

fn load_store(sidecar: &Path) -> Result<ExecProcessRegistryStore, String> {
    if !sidecar.exists() {
        return Ok(ExecProcessRegistryStore {
            schema_version: SCHEMA_VERSION.to_string(),
            revision: 0,
            entries: vec![],
            audit_events: vec![],
            warnings: vec![],
        });
    }
    let text = fs::read_to_string(sidecar).map_err(|error| {
        format!(
            "读取执行进程登记 sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    let store: ExecProcessRegistryStore = serde_json::from_str(&text).map_err(|error| {
        format!(
            "执行进程登记 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    if store.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "执行进程登记 schema 不匹配，已拒绝覆盖 {}",
            sidecar.display()
        ));
    }
    Ok(store)
}

fn write_store_atomic(sidecar: &Path, store: &ExecProcessRegistryStore) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("执行进程登记 sidecar 没有父目录：{}", sidecar.display()))?;
    let temp_path = parent.join(format!(
        ".exec-process-registry.{}.{}.tmp",
        timestamp_ms(),
        std::process::id()
    ));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("执行进程登记序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!(
                "创建执行进程登记临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!(
                "写入执行进程登记临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "同步执行进程登记临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换执行进程登记 sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    Ok(())
}

fn ps_field(pid: u32, field: &str) -> Result<Option<String>, String> {
    let output = Command::new("/bin/ps")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg(format!("{field}="))
        .output()
        .map_err(|error| format!("读取 PID {pid} 的 {field} 失败：{error}"))?;
    classify_ps_field_output(
        pid,
        field,
        output.status.success(),
        &output.stdout,
        &output.stderr,
    )
}

fn classify_ps_field_output(
    pid: u32,
    field: &str,
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<Option<String>, String> {
    if !success {
        let stderr = String::from_utf8_lossy(stderr);
        if (stdout.is_empty() && stderr.trim().is_empty())
            || stderr.contains("No such process")
            || stderr.contains("not found")
        {
            return Ok(None);
        }
        return Err(format!("读取 PID {pid} 的 {field} 失败：{}", stderr.trim()));
    }
    let value = String::from_utf8_lossy(stdout).trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

fn is_workbench_codex_exec(cmdline: &str) -> bool {
    cmdline.contains("codex exec")
}

fn same_process(entry: &RegisteredProcess, observed: &ObservedProcess) -> bool {
    entry.started_at == observed.started_at
        && entry.cmdline_summary == observed.cmdline
        && (!entry.process_group || entry.process_group_id == observed.process_group_id)
}

fn registry_audit(event_type: &str, entry: &RegisteredProcess, reason: &str) -> RegistryAuditEvent {
    let created_at = timestamp_ms().to_string();
    RegistryAuditEvent {
        event_id: format!("audit:{event_type}:{}:{created_at}", entry.pid),
        event_type: event_type.to_string(),
        created_at,
        reason: reason.to_string(),
    }
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("执行进程登记已被占用 {}：{error}", path.display()))?;
        file.write_all(write_id.as_bytes())
            .map_err(|error| format!("写入执行进程登记锁失败 {}：{error}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    struct FakeProcessOperations {
        processes: RefCell<BTreeMap<u32, ObservedProcess>>,
        killed: RefCell<Vec<(u32, bool)>>,
    }

    impl FakeProcessOperations {
        fn with_process(pid: u32, observed: ObservedProcess) -> Self {
            Self {
                processes: RefCell::new(BTreeMap::from([(pid, observed)])),
                killed: RefCell::new(vec![]),
            }
        }
    }

    impl ProcessOperations for FakeProcessOperations {
        fn inspect(&self, pid: u32) -> Result<Option<ObservedProcess>, String> {
            Ok(self.processes.borrow().get(&pid).cloned())
        }

        fn kill_and_wait(&self, pid: u32, process_group: bool) -> Result<(), String> {
            self.killed.borrow_mut().push((pid, process_group));
            self.processes.borrow_mut().remove(&pid);
            Ok(())
        }
    }

    fn test_workflow_state_path(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("exec-process-registry-{name}-{}", timestamp_ms()));
        fs::create_dir_all(&path).expect("test sidecar dir");
        path.join("workflow-state.v0.json")
    }

    fn entry(pid: u32) -> RegisteredProcess {
        RegisteredProcess {
            pid,
            run_id: "new_session:consult-readonly:abc".to_string(),
            started_at: "Fri Jul 10 21:49:12 2026".to_string(),
            cmdline_summary: "/tmp/codex exec -C /tmp/test --sandbox read-only".to_string(),
            process_group: false,
            process_group_id: None,
        }
    }

    fn observed_for(entry: &RegisteredProcess) -> ObservedProcess {
        ObservedProcess {
            started_at: entry.started_at.clone(),
            cmdline: entry.cmdline_summary.clone(),
            process_group_id: entry.process_group_id,
        }
    }

    #[test]
    fn durable_process_run_id_remains_traceable_when_prompt_ref_is_empty() {
        let first = durable_process_run_id(
            "resume",
            &["", "work-item:3b", "thread:3b", "/tmp/mario test"],
        );
        let replay = durable_process_run_id(
            "resume",
            &["", "work-item:3b", "thread:3b", "/tmp/mario test"],
        );
        let other = durable_process_run_id(
            "resume",
            &["", "work-item:other", "thread:3b", "/tmp/mario test"],
        );

        assert!(first.starts_with("codex-local:resume:"));
        assert_ne!(first, "resume:");
        assert_eq!(first, replay);
        assert_ne!(first, other);
    }

    #[test]
    fn registry_registers_and_unregisters_normal_completion() {
        let path = test_workflow_state_path("register-unregister");
        let registered = entry(41);
        register_entry(&path, registered.clone()).expect("register");
        let sidecar = sidecar_path(&path).expect("sidecar");
        assert_eq!(
            load_store(&sidecar).unwrap().entries,
            vec![registered.clone()]
        );

        unregister_entry(&path, &registered).expect("unregister");
        assert!(load_store(&sidecar).unwrap().entries.is_empty());
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn startup_reaps_only_a_registered_matching_orphan() {
        let path = test_workflow_state_path("reap");
        let registered = entry(42);
        register_entry(&path, registered.clone()).expect("register");
        let processes = FakeProcessOperations::with_process(42, observed_for(&registered));

        assert_eq!(reap_registered_orphans_with(&path, &processes).unwrap(), 1);
        assert_eq!(*processes.killed.borrow(), vec![(42, false)]);
        let store = load_store(&sidecar_path(&path).unwrap()).unwrap();
        assert!(store.entries.is_empty());
        assert!(store.audit_events.iter().any(|event| {
            event.event_type == "exec_process_registry_orphan_reaped"
                && event.reason.contains("回收了上次遗留的执行进程")
        }));
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn strict_manual_relay_registration_requires_and_reaps_the_exact_process_group() {
        let path = test_workflow_state_path("manual-relay-group");
        let pid = 52;
        let registered = RegisteredProcess {
            process_group: true,
            process_group_id: Some(pid),
            ..entry(pid)
        };
        let processes = FakeProcessOperations::with_process(pid, observed_for(&registered));

        let registration = register_codex_process_group_with(
            &path,
            "manual-relay-attempt:test",
            pid,
            "manual relay",
            &processes,
        )
        .expect("matching process group must register strictly");
        let sidecar = sidecar_path(&path).unwrap();
        let store = load_store(&sidecar).unwrap();
        assert_eq!(store.entries.len(), 1);
        assert!(store.entries[0].process_group);
        assert_eq!(store.entries[0].process_group_id, Some(pid));

        drop(registration);
        assert_eq!(
            load_store(&sidecar).unwrap().entries.len(),
            1,
            "durable registration must survive an abnormal handle drop"
        );
        assert_eq!(reap_registered_orphans_with(&path, &processes).unwrap(), 1);
        assert_eq!(*processes.killed.borrow(), vec![(pid, true)]);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn strict_manual_relay_registration_rejects_a_non_leader_pid() {
        let path = test_workflow_state_path("manual-relay-not-leader");
        let pid = 53;
        let mut observed = observed_for(&entry(pid));
        observed.process_group_id = Some(7);
        let processes = FakeProcessOperations::with_process(pid, observed);

        let error = register_codex_process_group_with(
            &path,
            "manual-relay-attempt:test",
            pid,
            "manual relay",
            &processes,
        )
        .err()
        .expect("non leader PID must be rejected");
        assert!(error.contains("不是已核验的进程组组长"));
        assert!(!sidecar_path(&path).unwrap().exists());
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn legacy_registered_process_without_group_fields_stays_single_pid() {
        let legacy: RegisteredProcess = serde_json::from_value(serde_json::json!({
            "pid": 54,
            "run_id": "legacy",
            "started_at": "Fri Jul 10 21:49:12 2026",
            "cmdline_summary": "/tmp/codex exec -C /tmp/test"
        }))
        .unwrap();
        assert!(!legacy.process_group);
        assert_eq!(legacy.process_group_id, None);
    }

    #[test]
    fn missing_pid_with_empty_ps_output_is_not_an_inspection_error() {
        assert_eq!(
            classify_ps_field_output(999_999, "command", false, b"", b"").unwrap(),
            None
        );
    }

    #[test]
    fn startup_never_kills_an_unregistered_pid() {
        let path = test_workflow_state_path("unregistered");
        let processes = FakeProcessOperations::with_process(77, observed_for(&entry(77)));

        assert_eq!(reap_registered_orphans_with(&path, &processes).unwrap(), 0);
        assert!(processes.killed.borrow().is_empty());
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn pid_reuse_identity_mismatch_only_unregisters() {
        let path = test_workflow_state_path("pid-reuse");
        let registered = entry(88);
        register_entry(&path, registered.clone()).expect("register");
        let processes = FakeProcessOperations::with_process(
            88,
            ObservedProcess {
                started_at: "Sat Jul 11 09:54:09 2026".to_string(),
                cmdline: registered.cmdline_summary.clone(),
                process_group_id: None,
            },
        );

        assert_eq!(reap_registered_orphans_with(&path, &processes).unwrap(), 0);
        assert!(processes.killed.borrow().is_empty());
        let store = load_store(&sidecar_path(&path).unwrap()).unwrap();
        assert!(store.entries.is_empty());
        assert!(store
            .audit_events
            .iter()
            .any(|event| event.event_type == "exec_process_registry_identity_mismatch"));
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
