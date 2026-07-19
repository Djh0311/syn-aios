// S1B project-scoped supervisor transport.
//
// A supervisor thread and its private CODEX_HOME are durable project state, but
// the `codex exec` process is deliberately one-shot.  There is no in-memory
// resident host, no daemon to reattach after restart, and no home cleanup on a
// normal conversation turn.

use std::io::{BufRead, BufReader};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

const SUPERVISOR_RESIDENT_DEVELOPER_INSTRUCTIONS: &str = "你处于项目主管会话。当前会话只能只读：不得修改文件、不得运行会改变项目状态的命令、不得扩大权限或绕过确认。请像正常对话一样回应用户；不要输出或要求固定 JSON 回合协议，也不要冒充用户。只有在你已形成完整终版方案时才调用私有 submit_proposal 工具落卡；纯文字方案、建议或问题都不会落卡、更不会推进工作流。工具未落卡时只用自然人话说明卡未生成，不复述工具参数、stderr 或内部诊断。批准卡之外不得启动链、派发 worker 或改变执行状态。";
const SUPERVISOR_RESIDENT_WATCHDOG_SILENCE: Duration = Duration::from_secs(120);
const SUPERVISOR_RESIDENT_WATCHDOG_POLL: Duration = Duration::from_millis(100);
const SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE: &str = "主管这句没接上——再发一次或换个说法。";
const SUPERVISOR_RESIDENT_HOME_CATEGORY: &str = "supervisor-resident-homes";
const SUPERVISOR_RESIDENT_HOME_ACTIVE: &str = "active";
const SUPERVISOR_RESIDENT_HOME_ARCHIVE: &str = "archive";
// Reuse the established metadata kind inside this separate private-home tree;
// the file's directory remains the S1B project-scoped owner boundary.
const SUPERVISOR_RESIDENT_HOME_METADATA: &str = SUPERVISOR_TEMP_HOME_METADATA;

// A user-originated message has a short critical section: persist the canonical
// event, bind the thread before tool calls, then append injection/reply events.
static SUPERVISOR_RESIDENT_CONVERSATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SupervisorResidentTurn {
    pub(crate) thread_id: String,
    pub(crate) content: String,
    recoverable_error_details: Vec<String>,
}

#[derive(Clone, Debug)]
struct SupervisorResidentOneShotPlan {
    command_plan: SupervisorCommandPlan,
    prompt: String,
    expected_thread_id: Option<String>,
    workflow_state_path: PathBuf,
    run_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SupervisorResidentOneShotFailure {
    ThreadInvalid(String),
    InvalidResume {
        exit_code: i32,
        stderr_path: PathBuf,
        stderr_detail: String,
    },
    WatchdogSilence,
    CleanupFailed(String),
    Protocol(String),
}

impl SupervisorResidentOneShotFailure {
    fn is_invalid_resume(&self) -> bool {
        matches!(self, Self::ThreadInvalid(_) | Self::InvalidResume { .. })
    }

    fn into_error(self) -> String {
        match self {
            Self::ThreadInvalid(_) | Self::InvalidResume { .. } => {
                SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE.to_string()
            }
            Self::CleanupFailed(detail) | Self::Protocol(detail) => detail,
            Self::WatchdogSilence => "supervisor_resident_watchdog_silence".to_string(),
        }
    }
}

trait SupervisorResidentOneShotRunner: Send + Sync {
    fn run(
        &self,
        plan: &SupervisorResidentOneShotPlan,
        home: &SupervisorResidentHome,
        on_turn_prepared: &mut dyn FnMut(u32) -> Result<(), String>,
        on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
    ) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure>;
}

struct RealSupervisorResidentOneShotRunner;

impl SupervisorResidentOneShotRunner for RealSupervisorResidentOneShotRunner {
    fn run(
        &self,
        plan: &SupervisorResidentOneShotPlan,
        home: &SupervisorResidentHome,
        on_turn_prepared: &mut dyn FnMut(u32) -> Result<(), String>,
        on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
    ) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
        run_real_supervisor_resident_oneshot(plan, home, on_turn_prepared, on_thread_started)
    }
}

#[derive(Clone, Debug)]
struct SupervisorResidentHome {
    root: PathBuf,
}

impl SupervisorResidentHome {
    fn root(&self) -> &Path {
        &self.root
    }
}

#[derive(Clone, Debug)]
struct SupervisorResidentHomeManager {
    base: PathBuf,
    auth_source: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SupervisorResidentHomeMetadata {
    run_id: String,
    workflow_state_path: PathBuf,
    generation: u64,
}

impl SupervisorResidentHomeManager {
    fn for_project(workflow_state_path: &Path, run_id: &str) -> Result<Self, String> {
        Ok(Self {
            base: crate::utils::store_paths::runtime_artifact_dir(
                workflow_state_path,
                SUPERVISOR_RESIDENT_HOME_CATEGORY,
                run_id,
            )?,
            auth_source: default_codex_auth_path()?,
        })
    }

    fn active_path(&self) -> PathBuf {
        self.base.join(SUPERVISOR_RESIDENT_HOME_ACTIVE)
    }

    fn ensure_active(
        &self,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
        generation: u64,
    ) -> Result<SupervisorResidentHome, String> {
        fs::create_dir_all(&self.base).map_err(|error| {
            format!(
                "创建项目主管私有 CODEX_HOME 根目录失败 {}：{error}",
                self.base.display()
            )
        })?;
        restrict_private_dir(&self.base)?;
        let active = self.active_path();
        if active.exists() {
            self.validate_existing_active(&active, plan, config, Some(generation))?;
            return Ok(SupervisorResidentHome { root: active });
        }
        self.create_active(plan, config, generation)
    }

    fn replace_active(
        &self,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
        generation: u64,
    ) -> Result<(SupervisorResidentHome, PathBuf), String> {
        fs::create_dir_all(&self.base).map_err(|error| {
            format!(
                "创建项目主管私有 CODEX_HOME 根目录失败 {}：{error}",
                self.base.display()
            )
        })?;
        restrict_private_dir(&self.base)?;
        let active = self.active_path();
        if !active.exists() {
            return Ok((
                self.create_active(plan, config, generation)?,
                self.base.clone(),
            ));
        }
        self.validate_existing_active(&active, plan, config, generation.checked_sub(1))?;
        let archive_root = self.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE);
        fs::create_dir_all(&archive_root).map_err(|error| {
            format!(
                "创建项目主管私有 home 归档目录失败 {}：{error}",
                archive_root.display()
            )
        })?;
        restrict_private_dir(&archive_root)?;
        let archive = archive_root.join(format!(
            "generation-{}-{}",
            generation.saturating_sub(1),
            crate::unix_timestamp_nanos()
        ));
        fs::rename(&active, &archive).map_err(|error| {
            format!(
                "归档待替换项目主管私有 home 失败 {}：{error}",
                active.display()
            )
        })?;
        match self.create_active(plan, config, generation) {
            Ok(home) => Ok((home, archive)),
            Err(error) => {
                // `create_active` only promotes a fully initialized staging
                // directory, so `active` is still vacant here. Restore the
                // prior durable home rather than deleting either copy in-turn.
                if let Err(restore_error) = fs::rename(&archive, &active) {
                    return Err(format!(
                        "项目主管私有 home 换代失败且旧 home 恢复失败：{error}; {restore_error}"
                    ));
                }
                Err(error)
            }
        }
    }

    fn create_active(
        &self,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
        generation: u64,
    ) -> Result<SupervisorResidentHome, String> {
        let active = self.active_path();
        if active.exists() {
            return Err(format!(
                "项目主管私有 CODEX_HOME 已存在，拒绝覆盖 {}",
                active.display()
            ));
        }
        // A partially initialized whitelist/auth home must never become active.
        // If creation fails during replacement, the old active home can then be
        // restored without deleting either generation.
        let staging = self.base.join(format!(
            ".active-staging-{generation}-{}",
            crate::unix_timestamp_nanos()
        ));
        self.create_home_at(&staging, plan, config, generation)?;
        fs::rename(&staging, &active).map_err(|error| {
            format!(
                "提升项目主管私有 CODEX_HOME 失败 {}：{error}",
                active.display()
            )
        })?;
        Ok(SupervisorResidentHome { root: active })
    }

    fn create_home_at(
        &self,
        home: &Path,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
        generation: u64,
    ) -> Result<(), String> {
        let auth_metadata = fs::metadata(&self.auth_source).map_err(|error| {
            format!(
                "项目主管需要读取既有 ~/.codex/auth.json 作为符号链接来源，但当前不可用 {}：{error}",
                self.auth_source.display()
            )
        })?;
        if !auth_metadata.is_file() {
            return Err("项目主管拒绝使用非普通文件的 ~/.codex/auth.json".to_string());
        }
        fs::create_dir(home).map_err(|error| {
            format!(
                "创建项目主管私有 CODEX_HOME 失败 {}：{error}",
                home.display()
            )
        })?;
        restrict_private_dir(home)?;
        let config_toml = supervisor_resident_mcp_config_toml(plan)?;
        write_private_file(
            &home.join(SUPERVISOR_TEMP_HOME_CONFIG),
            config_toml.as_bytes(),
        )?;
        let metadata = serde_json::to_vec(&SupervisorResidentHomeMetadata {
            run_id: config.run_id.clone(),
            workflow_state_path: config
                .supervisor_workflow_state_path
                .clone()
                .ok_or_else(|| "项目主管私有 home 缺 workflow state 路径".to_string())?,
            generation,
        })
        .map_err(|error| format!("序列化项目主管私有 home 元数据失败：{error}"))?;
        write_private_file(&home.join(SUPERVISOR_RESIDENT_HOME_METADATA), &metadata)?;
        create_auth_symlink(&self.auth_source, &home.join(SUPERVISOR_TEMP_HOME_AUTH))
    }

    fn validate_existing_active(
        &self,
        active: &Path,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
        expected_generation: Option<u64>,
    ) -> Result<(), String> {
        let metadata = fs::symlink_metadata(active).map_err(|error| {
            format!(
                "读取项目主管私有 CODEX_HOME 失败 {}：{error}",
                active.display()
            )
        })?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err("项目主管私有 CODEX_HOME 不是受控目录，已拒绝复用".to_string());
        }
        #[cfg(unix)]
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(
                "项目主管私有 CODEX_HOME 目录权限不属于 owner-only，已拒绝复用".to_string(),
            );
        }
        let config_path = active.join(SUPERVISOR_TEMP_HOME_CONFIG);
        let home_metadata = active.join(SUPERVISOR_RESIDENT_HOME_METADATA);
        let auth = active.join(SUPERVISOR_TEMP_HOME_AUTH);
        if !config_path.is_file() || !home_metadata.is_file() {
            return Err("项目主管私有 CODEX_HOME 缺白名单配置或元数据，已拒绝复用".to_string());
        }
        #[cfg(unix)]
        for private_file in [&config_path, &home_metadata] {
            let metadata = fs::metadata(private_file).map_err(|error| {
                format!(
                    "读取项目主管私有文件失败 {}：{error}",
                    private_file.display()
                )
            })?;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(format!(
                    "项目主管私有文件权限不属于 owner-only，已拒绝复用 {}",
                    private_file.display()
                ));
            }
        }
        let expected_config = supervisor_resident_mcp_config_toml(plan)?;
        let legacy_config = supervisor_mcp_config_toml(plan)?;
        let actual_config = fs::read_to_string(&config_path).map_err(|error| {
            format!(
                "读取项目主管私有 MCP 白名单失败 {}：{error}",
                config_path.display()
            )
        })?;
        let actual_config: toml::Value = toml::from_str(&actual_config)
            .map_err(|error| format!("项目主管私有 MCP 白名单格式损坏：{error}"))?;
        let expected_config: toml::Value = toml::from_str(&expected_config)
            .map_err(|error| format!("项目主管预期 MCP 白名单格式损坏：{error}"))?;
        let legacy_config: toml::Value = toml::from_str(&legacy_config)
            .map_err(|error| format!("项目主管旧版 MCP 白名单格式损坏：{error}"))?;
        let needs_legacy_config_migration = actual_config == legacy_config;
        if actual_config != expected_config && !needs_legacy_config_migration {
            return Err(
                "项目主管私有 CODEX_HOME MCP 白名单与当前受控计划不一致，已拒绝复用".to_string(),
            );
        }
        let home_metadata_bytes = fs::read(&home_metadata).map_err(|error| {
            format!(
                "读取项目主管私有 home 元数据失败 {}：{error}",
                home_metadata.display()
            )
        })?;
        let home_metadata: SupervisorResidentHomeMetadata =
            serde_json::from_slice(&home_metadata_bytes)
                .map_err(|error| format!("项目主管私有 home 元数据损坏：{error}"))?;
        if home_metadata.run_id != config.run_id
            || config
                .supervisor_workflow_state_path
                .as_ref()
                .is_none_or(|path| path != &home_metadata.workflow_state_path)
            || home_metadata.generation == 0
            || expected_generation.is_some_and(|generation| home_metadata.generation != generation)
        {
            return Err("项目主管私有 home 元数据与当前项目身份不一致，已拒绝复用".to_string());
        }
        let auth_metadata = fs::symlink_metadata(&auth).map_err(|error| {
            format!(
                "读取项目主管 auth.json 入口失败 {}：{error}",
                auth.display()
            )
        })?;
        if !auth_metadata.file_type().is_symlink() {
            return Err(
                "项目主管 auth.json 必须是既有认证文件的符号链接，拒绝复制 token".to_string(),
            );
        }
        if fs::read_link(&auth).map_err(|error| {
            format!(
                "读取项目主管 auth.json 符号链接失败 {}：{error}",
                auth.display()
            )
        })? != self.auth_source
        {
            return Err("项目主管 auth.json 未指向既有认证文件，拒绝复用".to_string());
        }
        if needs_legacy_config_migration {
            // An exact, already-validated resident config from before H2 may
            // gain the one explicitly authorized tool.  Any other drift was
            // rejected above, so this cannot become a generic config rewrite.
            replace_private_resident_config(
                &config_path,
                &supervisor_resident_mcp_config_toml(plan)?,
            )?;
        }
        Ok(())
    }
}

fn supervisor_resident_mcp_config_toml(plan: &SupervisorCommandPlan) -> Result<String, String> {
    let base = supervisor_mcp_config_toml(plan)?;
    let mut root: toml::Value =
        toml::from_str(&base).map_err(|error| format!("解析项目主管私有 MCP 配置失败：{error}"))?;
    let server = root
        .get_mut("mcp_servers")
        .and_then(toml::Value::as_table_mut)
        .and_then(|servers| servers.get_mut("supervisor_orchestrator"))
        .and_then(toml::Value::as_table_mut)
        .ok_or_else(|| "项目主管私有 MCP 配置缺 supervisor_orchestrator".to_string())?;
    server.insert(
        "enabled_tools".to_string(),
        toml::Value::Array(vec![toml::Value::String("submit_proposal".to_string())]),
    );
    let mut submit_proposal = toml::map::Map::new();
    submit_proposal.insert(
        "approval_mode".to_string(),
        toml::Value::String("approve".to_string()),
    );
    let mut tools = toml::map::Map::new();
    tools.insert(
        "submit_proposal".to_string(),
        toml::Value::Table(submit_proposal),
    );
    server.insert("tools".to_string(), toml::Value::Table(tools));
    toml::to_string(&root).map_err(|error| format!("序列化项目主管单工具 MCP 配置失败：{error}"))
}

fn replace_private_resident_config(path: &Path, contents: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("读取待迁移项目主管私有配置失败 {}：{error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err("项目主管私有 MCP 配置不是受控普通文件，拒绝迁移".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "项目主管私有 MCP 配置缺父目录，拒绝迁移".to_string())?;
    let replacement = parent.join(format!(
        ".config-h2-migration-{}",
        crate::unix_timestamp_nanos()
    ));
    write_private_file(&replacement, contents.as_bytes())?;
    if let Err(error) = fs::rename(&replacement, path) {
        let _ = fs::remove_file(&replacement);
        return Err(format!(
            "原子升级项目主管单工具 MCP 配置失败 {}：{error}",
            path.display()
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
enum SupervisorResidentLaunch {
    Created,
    Reused,
    Replaced { reason: String },
}

pub(crate) fn consult_supervisor_resident(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<String, String> {
    let _guard = resident_conversation_lock()
        .lock()
        .map_err(|_| "supervisor_resident_conversation_lock_poisoned".to_string())?;
    Ok(consult_supervisor_resident_turn_unlocked(
        workflow_state_path,
        project_root,
        workflow_id,
        prompt,
        prompt_kind,
    )?
    .content)
}

pub(crate) fn consult_supervisor_resident_turn(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<SupervisorResidentTurn, String> {
    let _guard = resident_conversation_lock()
        .lock()
        .map_err(|_| "supervisor_resident_conversation_lock_poisoned".to_string())?;
    consult_supervisor_resident_turn_unlocked(
        workflow_state_path,
        project_root,
        workflow_id,
        prompt,
        prompt_kind,
    )
}

fn consult_supervisor_resident_turn_unlocked(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<SupervisorResidentTurn, String> {
    if prompt_kind == "user_message" {
        return Err("supervisor_resident_user_message_requires_answer_command".to_string());
    }
    consult_supervisor_resident_with(
        &RealSupervisorResidentOneShotRunner,
        workflow_state_path,
        project_root,
        workflow_id,
        prompt,
        prompt_kind,
    )
}

fn consult_supervisor_resident_with(
    runner: &dyn SupervisorResidentOneShotRunner,
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<SupervisorResidentTurn, String> {
    validate_resident_request(project_root, prompt, prompt_kind)?;
    let run_id = resident_run_id(project_root);
    let config = resident_supervisor_config(workflow_state_path, &run_id);
    let home_manager = SupervisorResidentHomeManager::for_project(workflow_state_path, &run_id)?;
    consult_supervisor_resident_with_parts(
        runner,
        &home_manager,
        workflow_state_path,
        project_root,
        workflow_id,
        prompt,
        prompt_kind,
        &config,
        None,
    )
}

fn consult_supervisor_resident_with_parts(
    runner: &dyn SupervisorResidentOneShotRunner,
    home_manager: &SupervisorResidentHomeManager,
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
    config: &McpServerConfig,
    active_user_message_id: Option<&str>,
) -> Result<SupervisorResidentTurn, String> {
    // A failed TERM/KILL sweep is not a completed turn.  Reconcile its exact
    // process group before accepting another user message; if it is still
    // present, fail closed rather than spawning a second supervisor turn.
    reap_supervisor_resident_stale_sessions_at(workflow_state_path)?;
    if let Some(session) = supervisor_orchestrator::load_resident_turn_for_reconciliation(config)? {
        if session.launch_status == "resident_turn_cleanup_failed" {
            return Err(format!(
                "主管上一次回合的进程组尚未确认清理（pid={}）；已拒绝并发续聊，请重启工作台后重试。",
                session.host_pid
            ));
        }
    }
    let persisted = supervisor_orchestrator::load_resident_session(config)?;
    let executable = resident_workbench_executable()?;
    if let Some(session) = persisted.filter(|session| !session.thread_id.trim().is_empty()) {
        let command_plan = build_supervisor_resident_command_plan(
            project_root,
            workflow_state_path,
            &config.run_id,
            session.generation.max(1),
            &executable,
            Some(&session.thread_id),
        )?;
        let home = home_manager.ensure_active(&command_plan, config, session.generation.max(1))?;
        let plan = SupervisorResidentOneShotPlan {
            command_plan,
            prompt: prompt.to_string(),
            expected_thread_id: Some(session.thread_id.clone()),
            workflow_state_path: workflow_state_path.to_path_buf(),
            run_id: config.run_id.clone(),
        };
        match run_supervisor_resident_with_watchdog_retry(
            runner,
            &plan,
            &home,
            config,
            project_root,
            workflow_id,
            prompt_kind,
            active_user_message_id,
            session.generation.max(1),
            SupervisorResidentLaunch::Reused,
        ) {
            Err(error) if error.is_invalid_resume() => {
                // A resume rejection is allowed exactly one recovery path:
                // archive the former project home, rebuild facts, and make one
                // fresh initial exec.  The recovery call is deliberately not
                // matched again, so it cannot recurse.
                (|| {
                    let generation = session.generation.saturating_add(1).max(1);
                    let replacement_plan = build_supervisor_resident_command_plan(
                        project_root,
                        workflow_state_path,
                        &config.run_id,
                        generation,
                        &executable,
                        None,
                    )?;
                    let (replacement_home, _archive) =
                        home_manager.replace_active(&replacement_plan, config, generation)?;
                    let facts =
                        resident_rebuild_facts(workflow_state_path, project_root, workflow_id)?;
                    let opening = format!(
                        "{facts}\n\n===== 当前项目主管请求（{prompt_kind}）=====\n{prompt}"
                    );
                    let plan = SupervisorResidentOneShotPlan {
                        command_plan: replacement_plan,
                        prompt: opening,
                        expected_thread_id: None,
                        workflow_state_path: workflow_state_path.to_path_buf(),
                        run_id: config.run_id.clone(),
                    };
                    run_supervisor_resident_with_watchdog_retry(
                        runner,
                        &plan,
                        &replacement_home,
                        config,
                        project_root,
                        workflow_id,
                        prompt_kind,
                        active_user_message_id,
                        generation,
                        SupervisorResidentLaunch::Replaced {
                            reason: "invalid_resume".to_string(),
                        },
                    )
                    .map_err(SupervisorResidentOneShotFailure::into_error)
                })()
                .map_err(|_| SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE.to_string())
            }
            Ok(turn) => Ok(turn),
            Err(error) => Err(error.into_error()),
        }
    } else {
        let generation = 1;
        let command_plan = build_supervisor_resident_command_plan(
            project_root,
            workflow_state_path,
            &config.run_id,
            generation,
            &executable,
            None,
        )?;
        let home = home_manager.ensure_active(&command_plan, config, generation)?;
        let facts = resident_rebuild_facts(workflow_state_path, project_root, workflow_id)?;
        let plan = SupervisorResidentOneShotPlan {
            command_plan,
            prompt: format!("{facts}\n\n===== 当前项目主管请求（{prompt_kind}）=====\n{prompt}"),
            expected_thread_id: None,
            workflow_state_path: workflow_state_path.to_path_buf(),
            run_id: config.run_id.clone(),
        };
        run_supervisor_resident_with_watchdog_retry(
            runner,
            &plan,
            &home,
            config,
            project_root,
            workflow_id,
            prompt_kind,
            active_user_message_id,
            generation,
            SupervisorResidentLaunch::Created,
        )
        .map_err(SupervisorResidentOneShotFailure::into_error)
    }
}

#[allow(clippy::too_many_arguments)]
fn run_supervisor_resident_with_watchdog_retry(
    runner: &dyn SupervisorResidentOneShotRunner,
    plan: &SupervisorResidentOneShotPlan,
    home: &SupervisorResidentHome,
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    prompt_kind: &str,
    active_user_message_id: Option<&str>,
    generation: u64,
    launch: SupervisorResidentLaunch,
) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
    let mut attempt_plan = plan.clone();
    let mut attempt_launch = launch.clone();
    for attempt in 0..=1 {
        let mut bound_thread_id = None;
        let mut on_turn_prepared = |pid: u32| -> Result<(), String> {
            supervisor_orchestrator::record_resident_turn_prepared(
                config,
                project_root,
                workflow_id,
                pid,
                generation,
                active_user_message_id,
            )
        };
        let mut on_thread_started = |thread_id: &str, pid: u32| -> Result<(), String> {
            if thread_id.trim().is_empty() || pid == 0 {
                return Err("supervisor_resident_thread_started_identity_incomplete".to_string());
            }
            if let Some(expected) = attempt_plan.expected_thread_id.as_deref() {
                if expected != thread_id {
                    return Err("supervisor_resident_resume_thread_mismatch".to_string());
                }
            }
            if bound_thread_id.as_deref() == Some(thread_id) {
                return Ok(());
            }
            let result = match &attempt_launch {
                SupervisorResidentLaunch::Created => {
                    supervisor_orchestrator::record_resident_session_created(
                        config,
                        project_root,
                        workflow_id,
                        thread_id,
                        pid,
                        generation,
                    )
                }
                SupervisorResidentLaunch::Reused => {
                    supervisor_orchestrator::record_resident_session_reused(
                        config,
                        project_root,
                        workflow_id,
                        thread_id,
                        pid,
                        generation,
                    )
                }
                SupervisorResidentLaunch::Replaced { reason } => {
                    supervisor_orchestrator::record_resident_session_replaced(
                        config,
                        project_root,
                        workflow_id,
                        thread_id,
                        pid,
                        generation,
                        reason,
                    )
                }
            };
            result?;
            // The MCP server treats the prepared state as non-authorizing and
            // waits for this synchronous durable binding before accepting a
            // first-turn submit_proposal call.
            bound_thread_id = Some(thread_id.to_string());
            Ok(())
        };
        let result = runner.run(
            &attempt_plan,
            home,
            &mut on_turn_prepared,
            &mut on_thread_started,
        );
        match result {
            Ok(turn) => {
                if bound_thread_id.as_deref() != Some(turn.thread_id.as_str()) {
                    if let Err(error) = supervisor_orchestrator::record_resident_turn_exited(
                        config,
                        "thread_binding_missing_after_oneshot",
                    ) {
                        return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                            "supervisor_resident_thread_binding_missing_after_oneshot; supervisor_resident_lifecycle_exit_failed:{error}"
                        )));
                    }
                    return Err(SupervisorResidentOneShotFailure::Protocol(
                        "supervisor_resident_thread_binding_missing_after_oneshot".to_string(),
                    ));
                }
                if let (Some(message_id), Some(raw_detail)) = (
                    active_user_message_id,
                    turn.recoverable_error_details.first(),
                ) {
                    supervisor_orchestrator::record_resident_turn_recoverable_diagnostic(
                        config, message_id, raw_detail,
                    )
                    .map_err(|error| {
                        SupervisorResidentOneShotFailure::Protocol(format!(
                            "supervisor_resident_recoverable_error_audit_failed:{error}"
                        ))
                    })?;
                }
                if let Err(error) = supervisor_orchestrator::record_resident_consult_merged(
                    config,
                    project_root,
                    workflow_id,
                    prompt_kind,
                )
                .and_then(|_| {
                    supervisor_orchestrator::record_resident_turn_exited(config, "turn_completed")
                }) {
                    return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                        "supervisor_resident_audit_failed:{error}"
                    )));
                }
                return Ok(turn);
            }
            Err(SupervisorResidentOneShotFailure::WatchdogSilence) if attempt == 0 => {
                if let Err(error) = supervisor_orchestrator::record_resident_turn_exited(
                    config,
                    "watchdog_silence_retrying_once",
                ) {
                    return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                        "supervisor_resident_watchdog_silence; supervisor_resident_lifecycle_exit_failed:{error}"
                    )));
                }
                // If initial `exec` reached `thread.started` before becoming
                // silent, the retry must continue that exact thread rather
                // than opening a second initial conversation.
                if let Some(thread_id) = bound_thread_id.as_deref() {
                    attempt_plan = resume_supervisor_resident_retry_plan(
                        &attempt_plan,
                        project_root,
                        generation,
                        thread_id,
                    )?;
                    attempt_launch = SupervisorResidentLaunch::Reused;
                }
                continue;
            }
            Err(SupervisorResidentOneShotFailure::WatchdogSilence) => {
                if let Err(error) = supervisor_orchestrator::record_resident_turn_exited(
                    config,
                    "watchdog_silence_retry_exhausted",
                ) {
                    return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                        "{SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE}; supervisor_resident_lifecycle_exit_failed:{error}"
                    )));
                }
                return Err(SupervisorResidentOneShotFailure::Protocol(
                    SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE.to_string(),
                ));
            }
            Err(SupervisorResidentOneShotFailure::CleanupFailed(detail)) => {
                if let Err(lifecycle_error) =
                    supervisor_orchestrator::record_resident_turn_cleanup_failed(config, &detail)
                {
                    return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                        "{detail}; supervisor_resident_cleanup_lifecycle_failed:{lifecycle_error}"
                    )));
                }
                return Err(SupervisorResidentOneShotFailure::CleanupFailed(detail));
            }
            Err(error) => {
                let invalid_resume = error.is_invalid_resume();
                if invalid_resume {
                    if record_invalid_resume_failure(config, &error).is_err() {
                        return Err(SupervisorResidentOneShotFailure::Protocol(
                            SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE.to_string(),
                        ));
                    }
                }
                let reason = match &error {
                    SupervisorResidentOneShotFailure::ThreadInvalid(_)
                    | SupervisorResidentOneShotFailure::InvalidResume { .. } => "invalid_resume",
                    SupervisorResidentOneShotFailure::Protocol(_) => "turn_failed",
                    SupervisorResidentOneShotFailure::WatchdogSilence
                    | SupervisorResidentOneShotFailure::CleanupFailed(_) => unreachable!(),
                };
                if let Err(lifecycle_error) =
                    supervisor_orchestrator::record_resident_turn_exited(config, reason)
                {
                    if invalid_resume {
                        return Err(SupervisorResidentOneShotFailure::Protocol(
                            SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE.to_string(),
                        ));
                    }
                    return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                        "{}; supervisor_resident_lifecycle_exit_failed:{lifecycle_error}",
                        error.into_error()
                    )));
                }
                return Err(error);
            }
        }
    }
    unreachable!("watchdog retry loop has an explicit terminal branch")
}

fn resume_supervisor_resident_retry_plan(
    previous: &SupervisorResidentOneShotPlan,
    project_root: &str,
    generation: u64,
    thread_id: &str,
) -> Result<SupervisorResidentOneShotPlan, SupervisorResidentOneShotFailure> {
    let command_plan = build_supervisor_resident_command_plan(
        project_root,
        &previous.workflow_state_path,
        &previous.run_id,
        generation,
        &previous.command_plan.supervisor_mcp_command,
        Some(thread_id),
    )
    .map_err(SupervisorResidentOneShotFailure::Protocol)?;
    Ok(SupervisorResidentOneShotPlan {
        command_plan,
        prompt: previous.prompt.clone(),
        expected_thread_id: Some(thread_id.to_string()),
        workflow_state_path: previous.workflow_state_path.clone(),
        run_id: previous.run_id.clone(),
    })
}

fn run_real_supervisor_resident_oneshot(
    plan: &SupervisorResidentOneShotPlan,
    home: &SupervisorResidentHome,
    on_turn_prepared: &mut dyn FnMut(u32) -> Result<(), String>,
    on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
    let command_plan = &plan.command_plan;
    let output_dir = command_plan.stderr_path.parent().ok_or_else(|| {
        SupervisorResidentOneShotFailure::Protocol(
            "supervisor_resident_stderr_parent_missing".to_string(),
        )
    })?;
    fs::create_dir_all(output_dir).map_err(|error| {
        SupervisorResidentOneShotFailure::Protocol(format!(
            "supervisor_resident_output_dir_create_failed:{error}"
        ))
    })?;
    restrict_private_dir(output_dir).map_err(SupervisorResidentOneShotFailure::Protocol)?;
    match fs::remove_file(&command_plan.last_message_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                "supervisor_resident_last_message_reset_failed:{error}"
            )));
        }
    }
    let stderr_file = fs::File::create(&command_plan.stderr_path).map_err(|error| {
        SupervisorResidentOneShotFailure::Protocol(format!(
            "supervisor_resident_stderr_create_failed:{error}"
        ))
    })?;
    restrict_private_file(&command_plan.stderr_path)
        .map_err(SupervisorResidentOneShotFailure::Protocol)?;
    let mut command = Command::new(&command_plan.program);
    command
        .args(&command_plan.argv)
        .current_dir(&command_plan.current_dir)
        .env("CODEX_HOME", home.root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(stderr_file));
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn().map_err(|error| {
        SupervisorResidentOneShotFailure::Protocol(format!(
            "supervisor_resident_oneshot_spawn_failed:{error}"
        ))
    })?;
    let pid = child.id();
    if let Err(error) = on_turn_prepared(pid) {
        if let Err(cleanup_error) = stop_supervisor_resident_process_group(&mut child) {
            return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                "{error}; process_group_cleanup_failed:{cleanup_error}"
            )));
        }
        return Err(SupervisorResidentOneShotFailure::Protocol(error));
    }
    let registration = match crate::exec_process_registry::register_supervisor_oneshot_process_group(
        &plan.workflow_state_path,
        &plan.run_id,
        pid,
    ) {
        Ok(registration) => registration,
        Err(error) => {
            if let Err(cleanup_error) = stop_supervisor_resident_process_group(&mut child) {
                return Err(cleanup_failed(
                    format!("supervisor_resident_process_registration_failed:{error}"),
                    cleanup_error,
                ));
            }
            return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                "supervisor_resident_process_registration_failed:{error}"
            )));
        }
    };
    let mut bound_thread = None;
    if let Some(expected_thread_id) = plan.expected_thread_id.as_deref() {
        if let Err(error) = on_thread_started(expected_thread_id, pid) {
            if let Err(cleanup_error) =
                stop_and_unregister_supervisor_resident_process_group(&mut child, registration)
            {
                return Err(cleanup_failed(error, cleanup_error));
            }
            return Err(SupervisorResidentOneShotFailure::Protocol(error));
        }
        bound_thread = Some(expected_thread_id.to_string());
    }
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            if let Err(cleanup_error) =
                stop_and_unregister_supervisor_resident_process_group(&mut child, registration)
            {
                return Err(cleanup_failed(
                    "supervisor_resident_stdout_unavailable".to_string(),
                    cleanup_error,
                ));
            }
            return Err(SupervisorResidentOneShotFailure::Protocol(
                "supervisor_resident_stdout_unavailable".to_string(),
            ));
        }
    };
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if sender
                .send(
                    line.map_err(|error| format!("supervisor_resident_stdout_read_failed:{error}")),
                )
                .is_err()
            {
                break;
            }
        }
    });
    let mut stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            if let Err(cleanup_error) =
                stop_and_unregister_supervisor_resident_process_group(&mut child, registration)
            {
                return Err(cleanup_failed(
                    "supervisor_resident_stdin_unavailable".to_string(),
                    cleanup_error,
                ));
            }
            return Err(SupervisorResidentOneShotFailure::Protocol(
                "supervisor_resident_stdin_unavailable".to_string(),
            ));
        }
    };
    if let Err(error) = stdin
        .write_all(plan.prompt.as_bytes())
        .and_then(|_| stdin.write_all(b"\n"))
        .and_then(|_| stdin.flush())
    {
        if let Err(cleanup_error) =
            stop_and_unregister_supervisor_resident_process_group(&mut child, registration)
        {
            return Err(cleanup_failed(
                format!("supervisor_resident_stdin_write_failed:{error}"),
                cleanup_error,
            ));
        }
        return Err(SupervisorResidentOneShotFailure::Protocol(format!(
            "supervisor_resident_stdin_write_failed:{error}"
        )));
    }
    drop(stdin);

    let mut last_activity = Instant::now();
    let mut turn_completed = false;
    let mut turn_failed = None;
    let mut recoverable_error_details = Vec::new();
    let mut assistant_message = None;
    // Resume pre-registers its known durable thread below.  Only a parsed
    // stdout `thread.started` event may set this flag: the pre-registration is
    // not evidence that Codex accepted the resume ticket.
    let mut thread_started_event_seen = false;
    let status = loop {
        if last_activity.elapsed() >= SUPERVISOR_RESIDENT_WATCHDOG_SILENCE {
            if let Err(cleanup_error) =
                stop_and_unregister_supervisor_resident_process_group(&mut child, registration)
            {
                return Err(cleanup_failed(
                    "supervisor_resident_watchdog_silence".to_string(),
                    cleanup_error,
                ));
            }
            return Err(SupervisorResidentOneShotFailure::WatchdogSilence);
        }
        match receiver.recv_timeout(SUPERVISOR_RESIDENT_WATCHDOG_POLL) {
            Ok(Ok(line)) => {
                last_activity = Instant::now();
                if let Err(error) = apply_supervisor_resident_json_event(
                    &line,
                    pid,
                    &mut bound_thread,
                    &mut assistant_message,
                    &mut turn_completed,
                    &mut turn_failed,
                    &mut recoverable_error_details,
                    &mut thread_started_event_seen,
                    on_thread_started,
                ) {
                    if let Err(cleanup_error) =
                        stop_and_unregister_supervisor_resident_process_group(
                            &mut child,
                            registration,
                        )
                    {
                        return Err(cleanup_failed(error.into_error(), cleanup_error));
                    }
                    return Err(error);
                }
            }
            Ok(Err(error)) => {
                if let Err(cleanup_error) =
                    stop_and_unregister_supervisor_resident_process_group(&mut child, registration)
                {
                    return Err(cleanup_failed(error, cleanup_error));
                }
                return Err(SupervisorResidentOneShotFailure::Protocol(error));
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {}
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if let Err(error) = drain_supervisor_resident_events(
                    &receiver,
                    pid,
                    &mut bound_thread,
                    &mut assistant_message,
                    &mut turn_completed,
                    &mut turn_failed,
                    &mut recoverable_error_details,
                    &mut thread_started_event_seen,
                    on_thread_started,
                ) {
                    if let Err(cleanup_error) =
                        sweep_and_unregister_supervisor_resident_process_group(pid, registration)
                    {
                        return Err(cleanup_failed(error.into_error(), cleanup_error));
                    }
                    return Err(error);
                }
                break status;
            }
            Ok(None) => {}
            Err(error) => {
                if let Err(cleanup_error) =
                    stop_and_unregister_supervisor_resident_process_group(&mut child, registration)
                {
                    return Err(cleanup_failed(
                        format!("supervisor_resident_process_wait_failed:{error}"),
                        cleanup_error,
                    ));
                }
                return Err(SupervisorResidentOneShotFailure::Protocol(format!(
                    "supervisor_resident_process_wait_failed:{error}"
                )));
            }
        }
    };
    sweep_and_unregister_supervisor_resident_process_group(pid, registration).map_err(|error| {
        cleanup_failed(
            "supervisor_resident_process_group_cleanup_failed".to_string(),
            error,
        )
    })?;
    if !status.success() {
        let exit_code = status.code().unwrap_or(-1);
        if let Some(error) = classify_resume_exit_without_thread_started(
            plan,
            exit_code,
            thread_started_event_seen,
            command_plan.stderr_path.clone(),
            read_bounded_private_stderr(&command_plan.stderr_path),
        ) {
            return Err(error);
        }
    }
    finalize_supervisor_resident_turn(
        status.success(),
        status.code().unwrap_or(-1),
        turn_failed,
        turn_completed,
        thread_started_event_seen,
        bound_thread,
        fs::read_to_string(&command_plan.last_message_path)
            .ok()
            .filter(|value| !value.trim().is_empty()),
        assistant_message,
        recoverable_error_details,
    )
}

#[allow(clippy::too_many_arguments)]
fn finalize_supervisor_resident_turn(
    status_success: bool,
    exit_code: i32,
    turn_failed: Option<String>,
    turn_completed: bool,
    thread_started_event_seen: bool,
    bound_thread: Option<String>,
    last_message: Option<String>,
    assistant_message: Option<String>,
    recoverable_error_details: Vec<String>,
) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
    if let Some(error) = turn_failed {
        return Err(classify_supervisor_resident_failure(error));
    }
    if !status_success {
        return Err(SupervisorResidentOneShotFailure::Protocol(format!(
            "supervisor_resident_oneshot_exit_failed:{exit_code}"
        )));
    }
    if !turn_completed {
        return Err(SupervisorResidentOneShotFailure::Protocol(
            "supervisor_resident_turn_completed_event_missing".to_string(),
        ));
    }
    if !thread_started_event_seen {
        return Err(SupervisorResidentOneShotFailure::Protocol(
            "supervisor_resident_thread_started_event_missing".to_string(),
        ));
    }
    let thread_id = bound_thread.ok_or_else(|| {
        SupervisorResidentOneShotFailure::Protocol(
            "supervisor_resident_thread_started_event_missing".to_string(),
        )
    })?;
    let content = last_message.or(assistant_message).ok_or_else(|| {
        SupervisorResidentOneShotFailure::Protocol(
            "supervisor_resident_assistant_message_missing".to_string(),
        )
    })?;
    Ok(SupervisorResidentTurn {
        thread_id,
        content,
        recoverable_error_details,
    })
}

#[allow(clippy::too_many_arguments)]
fn drain_supervisor_resident_events(
    receiver: &Receiver<Result<String, String>>,
    pid: u32,
    bound_thread: &mut Option<String>,
    assistant_message: &mut Option<String>,
    turn_completed: &mut bool,
    turn_failed: &mut Option<String>,
    recoverable_error_details: &mut Vec<String>,
    thread_started_event_seen: &mut bool,
    on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
) -> Result<(), SupervisorResidentOneShotFailure> {
    loop {
        match receiver.try_recv() {
            Ok(Ok(line)) => apply_supervisor_resident_json_event(
                &line,
                pid,
                bound_thread,
                assistant_message,
                turn_completed,
                turn_failed,
                recoverable_error_details,
                thread_started_event_seen,
                on_thread_started,
            )?,
            Ok(Err(error)) => return Err(SupervisorResidentOneShotFailure::Protocol(error)),
            Err(mpsc::TryRecvError::Empty | mpsc::TryRecvError::Disconnected) => return Ok(()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_supervisor_resident_json_event(
    line: &str,
    pid: u32,
    bound_thread: &mut Option<String>,
    assistant_message: &mut Option<String>,
    turn_completed: &mut bool,
    turn_failed: &mut Option<String>,
    recoverable_error_details: &mut Vec<String>,
    thread_started_event_seen: &mut bool,
    on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
) -> Result<(), SupervisorResidentOneShotFailure> {
    let value: Value = serde_json::from_str(line).map_err(|error| {
        SupervisorResidentOneShotFailure::Protocol(format!(
            "supervisor_resident_stdout_json_invalid:{error}"
        ))
    })?;
    let event_type = value.get("type").and_then(Value::as_str).ok_or_else(|| {
        SupervisorResidentOneShotFailure::Protocol(
            "supervisor_resident_stdout_event_type_missing".to_string(),
        )
    })?;
    match event_type {
        "thread.started" => {
            let thread_id = value
                .get("thread_id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    SupervisorResidentOneShotFailure::Protocol(
                        "supervisor_resident_thread_started_id_missing".to_string(),
                    )
                })?;
            if let Some(current) = bound_thread.as_deref() {
                if current != thread_id {
                    return Err(SupervisorResidentOneShotFailure::Protocol(
                        "supervisor_resident_multiple_thread_ids_in_one_turn".to_string(),
                    ));
                }
            } else {
                on_thread_started(thread_id, pid)
                    .map_err(SupervisorResidentOneShotFailure::Protocol)?;
                *bound_thread = Some(thread_id.to_string());
            }
            *thread_started_event_seen = true;
        }
        "item.completed" => {
            if value.pointer("/item/type").and_then(Value::as_str) == Some("agent_message") {
                if let Some(text) = value.pointer("/item/text").and_then(Value::as_str) {
                    if !text.trim().is_empty() {
                        *assistant_message = Some(text.to_string());
                    }
                }
            }
        }
        "turn.completed" => *turn_completed = true,
        "turn.failed" | "error" => {
            let detail = value
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| value.pointer("/error/message").and_then(Value::as_str))
                .unwrap_or("codex turn failed");
            if event_type == "turn.failed" {
                *turn_failed = Some(detail.to_string());
            } else {
                recoverable_error_details.push(detail.to_string());
            }
        }
        _ => {}
    }
    Ok(())
}

fn classify_supervisor_resident_failure(detail: String) -> SupervisorResidentOneShotFailure {
    if resident_thread_invalid(&detail) {
        SupervisorResidentOneShotFailure::ThreadInvalid(detail)
    } else {
        SupervisorResidentOneShotFailure::Protocol(format!(
            "supervisor_resident_turn_failed:{detail}"
        ))
    }
}

fn classify_resume_exit_without_thread_started(
    plan: &SupervisorResidentOneShotPlan,
    exit_code: i32,
    thread_started_event_seen: bool,
    stderr_path: PathBuf,
    stderr_detail: String,
) -> Option<SupervisorResidentOneShotFailure> {
    (plan.expected_thread_id.is_some() && exit_code != 0 && !thread_started_event_seen).then_some(
        SupervisorResidentOneShotFailure::InvalidResume {
            exit_code,
            stderr_path,
            stderr_detail,
        },
    )
}

fn read_bounded_private_stderr(path: &Path) -> String {
    const MAX_AUDIT_STDERR_BYTES: usize = 4096;

    match fs::read_to_string(path) {
        Ok(stderr) if stderr.len() <= MAX_AUDIT_STDERR_BYTES => stderr,
        Ok(stderr) => {
            let cutoff = stderr
                .char_indices()
                .take_while(|(index, _)| *index < MAX_AUDIT_STDERR_BYTES)
                .last()
                .map(|(index, character)| index + character.len_utf8())
                .unwrap_or(0);
            format!(
                "{}\n[stderr truncated for private audit]",
                &stderr[..cutoff]
            )
        }
        Err(error) => format!("stderr_read_failed:{error}"),
    }
}

fn record_invalid_resume_failure(
    config: &McpServerConfig,
    failure: &SupervisorResidentOneShotFailure,
) -> Result<(), String> {
    match failure {
        SupervisorResidentOneShotFailure::InvalidResume {
            exit_code,
            stderr_path,
            stderr_detail,
        } => supervisor_orchestrator::record_resident_invalid_resume_detected(
            config,
            "resume_exit_without_thread_started",
            Some(*exit_code),
            false,
            Some(stderr_path),
            stderr_detail,
        ),
        SupervisorResidentOneShotFailure::ThreadInvalid(detail) => {
            supervisor_orchestrator::record_resident_invalid_resume_detected(
                config,
                "terminal_thread_invalid",
                None,
                false,
                None,
                detail,
            )
        }
        SupervisorResidentOneShotFailure::WatchdogSilence
        | SupervisorResidentOneShotFailure::CleanupFailed(_)
        | SupervisorResidentOneShotFailure::Protocol(_) => Ok(()),
    }
}

fn cleanup_failed(
    context: String,
    cleanup_error: impl std::fmt::Display,
) -> SupervisorResidentOneShotFailure {
    SupervisorResidentOneShotFailure::CleanupFailed(format!(
        "{context}; process_group_cleanup_failed:{cleanup_error}"
    ))
}

fn stop_supervisor_resident_process_group(child: &mut Child) -> Result<(), String> {
    let pid = child.id();
    #[cfg(unix)]
    {
        let _ = Command::new("/bin/kill")
            .arg("-TERM")
            .arg(format!("-{pid}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        for _ in 0..32 {
            if child
                .try_wait()
                .map_err(|error| format!("supervisor_resident_process_group_wait_failed:{error}"))?
                .is_some()
            {
                return sweep_supervisor_resident_process_group(pid);
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        sweep_supervisor_resident_process_group(pid)?;
    }
    let _ = child.kill();
    child
        .wait()
        .map_err(|error| format!("supervisor_resident_process_group_wait_failed:{error}"))?;
    sweep_supervisor_resident_process_group(pid)
}

fn sweep_supervisor_resident_process_group(pid: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        let _ = Command::new("/bin/kill")
            .arg("-KILL")
            .arg(format!("-{pid}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        for _ in 0..=32 {
            let status = Command::new("/bin/kill")
                .arg("-0")
                .arg(format!("-{pid}"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|error| {
                    format!("supervisor_resident_process_group_probe_failed:{error}")
                })?;
            if !status.success() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        return Err(format!(
            "supervisor_resident_process_group_still_alive_after_sweep:{pid}"
        ));
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        Ok(())
    }
}

fn stop_and_unregister_supervisor_resident_process_group(
    child: &mut Child,
    registration: crate::exec_process_registry::DurableProcessRegistration,
) -> Result<(), String> {
    stop_supervisor_resident_process_group(child)?;
    registration.unregister()
}

fn sweep_and_unregister_supervisor_resident_process_group(
    pid: u32,
    registration: crate::exec_process_registry::DurableProcessRegistration,
) -> Result<(), String> {
    sweep_supervisor_resident_process_group(pid)?;
    registration.unregister()
}

fn validate_resident_request(
    project_root: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<(), String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT {
        return Err(crate::legacy_product_command_blocked_message(
            "supervisor_resident_session",
        ));
    }
    if prompt.trim().is_empty() {
        return Err("supervisor_resident_prompt_empty".to_string());
    }
    if !matches!(
        prompt_kind,
        "project_consult" | "director_plan" | "director_plan_preview" | "user_message"
    ) {
        return Err("supervisor_resident_prompt_kind_not_allowed".to_string());
    }
    Ok(())
}

const SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT: &str =
    "supervisor_resident_user_message_recorded";
const SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT: &str =
    "supervisor_resident_user_message_injected";
const SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT: &str =
    "supervisor_resident_supervisor_message_recorded";

#[derive(serde::Deserialize)]
pub(crate) struct SubmitSupervisorResidentAnswerRequest {
    pub(crate) project_id: String,
    pub(crate) workflow_id: String,
    pub(crate) message_text: String,
    #[serde(default)]
    pub(crate) client_request_id: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct SupervisorResidentAnswerOutcome {
    pub(crate) status: String,
    pub(crate) reply_injected: bool,
    pub(crate) thread_id: Option<String>,
    pub(crate) supervisor_reply: Option<String>,
    pub(crate) message: String,
}

pub(crate) fn resident_conversation_lock() -> &'static Mutex<()> {
    SUPERVISOR_RESIDENT_CONVERSATION_LOCK.get_or_init(|| Mutex::new(()))
}

fn resident_event_string<'a>(event: &'a Value, field: &str) -> Option<&'a str> {
    event.get(field).and_then(Value::as_str)
}

fn valid_resident_client_request_id(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

fn resident_client_request_id(
    request: &SubmitSupervisorResidentAnswerRequest,
) -> Result<Option<&str>, String> {
    match request.client_request_id.as_deref() {
        Some(value) if valid_resident_client_request_id(value) => Ok(Some(value)),
        Some(_) => Err("supervisor_resident_client_request_id_invalid".to_string()),
        None => Ok(None),
    }
}

fn require_resident_workflow_binding(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
) -> Result<String, String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT {
        return Err(crate::legacy_product_command_blocked_message(
            "supervisor_resident_session",
        ));
    }
    if workflow_id.trim().is_empty() {
        return Err("supervisor_resident_workflow_id_required".to_string());
    }
    let project_id = crate::project_id(project_root);
    let snapshot = crate::read_workflow_state_snapshot(workflow_state_path)?;
    if snapshot.project_workflows.iter().any(|workflow| {
        workflow.project_id == project_id
            && workflow.project_root == project_root
            && workflow.workflow_id == workflow_id
    }) {
        Ok(project_id)
    } else {
        Err("supervisor_resident_project_workflow_binding_not_found".to_string())
    }
}

fn resident_message_target_ref(workflow_id: &str, message_id: &str) -> String {
    format!("{workflow_id}:resident-message:{message_id}")
}

fn append_resident_message_canonical_event(
    workflow_state_path: &Path,
    event: Value,
    phase: &str,
) -> Result<(), String> {
    let mut value = crate::read_workflow_state_value(workflow_state_path)?;
    crate::array_mut(&mut value, "audit_events")?.push(event);
    crate::write_m5b_batch2_workflow_state(workflow_state_path, phase, &value)
}

fn append_resident_user_message_recorded(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    message_id: &str,
    message_text: &str,
    client_request_id: Option<&str>,
) -> Result<(), String> {
    append_resident_message_canonical_event(
        workflow_state_path,
        json!({
            "event_id": format!("supervisor-resident-message:user:{}", crate::unix_timestamp_nanos()),
            "event_type": SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT,
            "target_ref": resident_message_target_ref(workflow_id, message_id),
            "project_id": project_id,
            "workflow_id": workflow_id,
            "message_id": message_id,
            "message_text": message_text,
            "client_request_id": client_request_id,
            "actor_ref": "user",
            "source_kind": "supervisor_resident_user_message",
            "permission_level": "read_only_conversation",
            "created_at": crate::unix_timestamp_string(),
            "reason": message_text,
        }),
        "supervisor_resident_user_message_recorded",
    )
}

fn resident_canonical_events(workflow_state_path: &Path) -> Result<Vec<Value>, String> {
    Ok(crate::read_workflow_state_value(workflow_state_path)?
        .get("audit_events")
        .and_then(Value::as_array)
        .ok_or_else(|| "supervisor_resident_audit_events_missing".to_string())?
        .clone())
}

fn is_recorded_resident_user_message(event: &Value, project_id: &str, workflow_id: &str) -> bool {
    resident_event_string(event, "event_type")
        == Some(SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT)
        && resident_event_string(event, "project_id") == Some(project_id)
        && resident_event_string(event, "workflow_id") == Some(workflow_id)
        && resident_event_string(event, "actor_ref") == Some("user")
        && resident_event_string(event, "source_kind") == Some("supervisor_resident_user_message")
}

fn find_recorded_resident_user_message(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    client_request_id: Option<&str>,
    message_id: Option<&str>,
) -> Result<Option<(String, String)>, String> {
    let mut found = None;
    for event in resident_canonical_events(workflow_state_path)? {
        if !is_recorded_resident_user_message(&event, project_id, workflow_id) {
            continue;
        }
        let matches = match (client_request_id, message_id) {
            (Some(client_request_id), None) => {
                resident_event_string(&event, "client_request_id") == Some(client_request_id)
            }
            (None, Some(message_id)) => {
                resident_event_string(&event, "message_id") == Some(message_id)
            }
            _ => false,
        };
        if !matches {
            continue;
        }
        let message_id = resident_event_string(&event, "message_id")
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "supervisor_resident_recorded_message_id_missing".to_string())?;
        let message_text = resident_event_string(&event, "message_text")
            .ok_or_else(|| "supervisor_resident_recorded_message_text_missing".to_string())?;
        if found
            .replace((message_id.to_string(), message_text.to_string()))
            .is_some()
        {
            return Err("supervisor_resident_client_request_id_ambiguous".to_string());
        }
    }
    Ok(found)
}

fn append_resident_user_message_injected(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    message_id: &str,
    thread_id: &str,
) -> Result<(), String> {
    append_resident_message_canonical_event(
        workflow_state_path,
        json!({
            "event_id": format!("supervisor-resident-message:injected:{}", crate::unix_timestamp_nanos()),
            "event_type": SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT,
            "target_ref": resident_message_target_ref(workflow_id, message_id),
            "project_id": project_id,
            "workflow_id": workflow_id,
            "message_id": message_id,
            "thread_id": thread_id,
            "actor_ref": "supervisor_resident",
            "source_kind": "supervisor_resident_user_message_injection",
            "permission_level": "read_only_conversation",
            "created_at": crate::unix_timestamp_string(),
            "reason": "用户消息已通过同一主管 thread 注入；未伪造用户输入。",
        }),
        "supervisor_resident_user_message_injected",
    )
}

fn append_resident_supervisor_message_recorded(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    reply_to_message_id: &str,
    thread_id: &str,
    message_text: &str,
    proposal_outcome: &str,
) -> Result<(), String> {
    let message_id = format!("supervisor:{}", crate::unix_timestamp_nanos());
    append_resident_message_canonical_event(
        workflow_state_path,
        json!({
            "event_id": format!("supervisor-resident-message:supervisor:{}", crate::unix_timestamp_nanos()),
            "event_type": SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT,
            "target_ref": resident_message_target_ref(workflow_id, &message_id),
            "project_id": project_id,
            "workflow_id": workflow_id,
            "message_id": message_id,
            "reply_to_message_id": reply_to_message_id,
            "thread_id": thread_id,
            "message_text": message_text,
            "proposal_outcome": proposal_outcome,
            "actor_ref": "supervisor_resident",
            "source_kind": "supervisor_resident_supervisor_message",
            "permission_level": "read_only_conversation",
            "created_at": crate::unix_timestamp_string(),
            "reason": message_text,
        }),
        "supervisor_resident_supervisor_message_recorded",
    )
}

fn recorded_resident_reply_outcome(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    message_id: &str,
    config: &McpServerConfig,
) -> Result<Option<SupervisorResidentAnswerOutcome>, String> {
    let events = resident_canonical_events(workflow_state_path)?;
    let injected = events.iter().rev().find(|event| {
        resident_event_string(event, "event_type")
            == Some(SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT)
            && resident_event_string(event, "project_id") == Some(project_id)
            && resident_event_string(event, "workflow_id") == Some(workflow_id)
            && resident_event_string(event, "message_id") == Some(message_id)
    });
    if let Some(reply) = events.iter().rev().find(|event| {
        resident_event_string(event, "event_type")
            == Some(SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT)
            && resident_event_string(event, "project_id") == Some(project_id)
            && resident_event_string(event, "workflow_id") == Some(workflow_id)
            && resident_event_string(event, "reply_to_message_id") == Some(message_id)
    }) {
        let thread_id = resident_event_string(reply, "thread_id")
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "supervisor_resident_recorded_reply_thread_missing".to_string())?;
        let content = resident_event_string(reply, "message_text")
            .ok_or_else(|| "supervisor_resident_recorded_reply_text_missing".to_string())?;
        let proposal_outcome = resident_event_string(reply, "proposal_outcome")
            .filter(|value| matches!(*value, "materialized" | "tool_failed" | "not_requested"))
            .map(str::to_string)
            .or_else(|| {
                supervisor_orchestrator::resident_active_proposal_outcome(config, message_id).ok()
            })
            .unwrap_or_else(|| "not_requested".to_string());
        return Ok(Some(supervisor_resident_answer_outcome(
            &proposal_outcome,
            thread_id,
            content,
        )));
    }
    if let Some(injected) = injected {
        return Ok(Some(SupervisorResidentAnswerOutcome {
            status: "message_recorded_supervisor_incomplete".to_string(),
            reply_injected: false,
            thread_id: resident_event_string(injected, "thread_id").map(str::to_string),
            supervisor_reply: None,
            message: "消息已送到主管，但主管这次没回上来——可以再发一次。".to_string(),
        }));
    }
    Ok(None)
}

fn supervisor_resident_answer_outcome(
    proposal_outcome: &str,
    thread_id: &str,
    content: &str,
) -> SupervisorResidentAnswerOutcome {
    let (status, message) = match proposal_outcome {
        "materialized" => (
            "message_sent_proposal_materialized",
            "用户消息已同 thread 注入；主管回复与待确认方案卡已写入。",
        ),
        "tool_failed" => (
            "message_sent_proposal_tool_failed",
            "主管收到了，但方案卡没有生成——请再说一次“出方案”。",
        ),
        _ => (
            "message_sent",
            "用户消息已同 thread 注入；主管回复已写入项目对话。",
        ),
    };
    SupervisorResidentAnswerOutcome {
        status: status.to_string(),
        reply_injected: true,
        thread_id: Some(thread_id.to_string()),
        supervisor_reply: Some(content.to_string()),
        message: message.to_string(),
    }
}

fn resident_user_message_prompt(message_text: &str) -> String {
    format!(
        "===== 用户通过工作台提交的原文（唯一用户输入）=====\n{message_text}\n===== 用户原文结束 =====\n\n请像正常对话一样直接继续同一 thread。可以提问、澄清、讨论或给建议；不要把自己的文字当作用户决定，也不要生成固定 JSON 回合。只有完整终版方案才调用 submit_proposal；工具成功前不要声称右侧已有方案卡。聊天本身绝不批准、起链或派发 worker。"
    )
}

fn resolve_resident_message_scope(
    workflow_state_path: &Path,
    request: &SubmitSupervisorResidentAnswerRequest,
) -> Result<String, String> {
    if request.project_id.trim().is_empty() || request.workflow_id.trim().is_empty() {
        return Err("supervisor_resident_message_identity_incomplete".to_string());
    }
    let project_root = crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
    let expected_project_id =
        require_resident_workflow_binding(workflow_state_path, project_root, &request.workflow_id)?;
    if request.project_id != expected_project_id {
        return Err("supervisor_resident_message_project_id_mismatch".to_string());
    }
    Ok(project_root.to_string())
}

fn submit_supervisor_resident_answer_with(
    runner: &dyn SupervisorResidentOneShotRunner,
    workflow_state_path: &Path,
    request: &SubmitSupervisorResidentAnswerRequest,
) -> Result<SupervisorResidentAnswerOutcome, String> {
    let project_root = crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
    let run_id = resident_run_id(project_root);
    let home_manager = SupervisorResidentHomeManager::for_project(workflow_state_path, &run_id)?;
    let config = resident_supervisor_config(workflow_state_path, &run_id);
    submit_supervisor_resident_answer_with_parts(
        runner,
        &home_manager,
        workflow_state_path,
        request,
        &config,
    )
}

fn submit_supervisor_resident_answer_with_parts(
    runner: &dyn SupervisorResidentOneShotRunner,
    home_manager: &SupervisorResidentHomeManager,
    workflow_state_path: &Path,
    request: &SubmitSupervisorResidentAnswerRequest,
    config: &McpServerConfig,
) -> Result<SupervisorResidentAnswerOutcome, String> {
    if request.message_text.trim().is_empty() {
        return Err("supervisor_resident_message_text_empty".to_string());
    }
    let client_request_id = resident_client_request_id(request)?;
    let project_root = resolve_resident_message_scope(workflow_state_path, request)?;
    let message_id = if let Some(client_request_id) = client_request_id {
        match find_recorded_resident_user_message(
            workflow_state_path,
            &request.project_id,
            &request.workflow_id,
            Some(client_request_id),
            None,
        )
        .map_err(|_| "supervisor_resident_message_delivery_unknown".to_string())?
        {
            Some((message_id, recorded_message_text)) => {
                if recorded_message_text != request.message_text {
                    return Err("supervisor_resident_client_request_reuse_mismatch".to_string());
                }
                if let Some(outcome) = recorded_resident_reply_outcome(
                    workflow_state_path,
                    &request.project_id,
                    &request.workflow_id,
                    &message_id,
                    config,
                )
                .map_err(|_| "supervisor_resident_message_delivery_unknown".to_string())?
                {
                    return Ok(outcome);
                }
                return Ok(SupervisorResidentAnswerOutcome {
                    status: "message_recorded_supervisor_incomplete".to_string(),
                    reply_injected: false,
                    thread_id: None,
                    supervisor_reply: None,
                    message: "消息已送到主管，但主管这次没回上来——可以再发一次。".to_string(),
                });
            }
            None => format!("user:{}", crate::unix_timestamp_nanos()),
        }
    } else {
        format!("user:{}", crate::unix_timestamp_nanos())
    };
    if append_resident_user_message_recorded(
        workflow_state_path,
        &request.project_id,
        &request.workflow_id,
        &message_id,
        &request.message_text,
        client_request_id,
    )
    .is_err()
        && find_recorded_resident_user_message(
            workflow_state_path,
            &request.project_id,
            &request.workflow_id,
            None,
            Some(&message_id),
        )
        .map_err(|_| "supervisor_resident_message_delivery_unknown".to_string())?
        .is_none()
    {
        return Ok(SupervisorResidentAnswerOutcome {
            status: "message_not_recorded".to_string(),
            reply_injected: false,
            thread_id: None,
            supervisor_reply: None,
            message: "这句没送到主管——稍后再试一次。".to_string(),
        });
    }
    let turn = match consult_supervisor_resident_with_parts(
        runner,
        home_manager,
        workflow_state_path,
        &project_root,
        &request.workflow_id,
        &resident_user_message_prompt(&request.message_text),
        "user_message",
        config,
        Some(&message_id),
    ) {
        Ok(turn) => turn,
        Err(_) => {
            return Ok(SupervisorResidentAnswerOutcome {
                status: "message_recorded_supervisor_incomplete".to_string(),
                reply_injected: false,
                thread_id: None,
                supervisor_reply: None,
                message: "消息已送到主管，但主管这次没回上来——可以再发一次。".to_string(),
            });
        }
    };
    if append_resident_user_message_injected(
        workflow_state_path,
        &request.project_id,
        &request.workflow_id,
        &message_id,
        &turn.thread_id,
    )
    .is_err()
    {
        return Ok(SupervisorResidentAnswerOutcome {
            status: "message_recorded_supervisor_incomplete".to_string(),
            reply_injected: false,
            thread_id: Some(turn.thread_id),
            supervisor_reply: None,
            message: "消息已送到主管，但主管这次没回上来——可以再发一次。".to_string(),
        });
    }
    let proposal_outcome =
        supervisor_orchestrator::resident_active_proposal_outcome(config, &message_id)
            .unwrap_or_else(|_| "not_requested".to_string());
    if append_resident_supervisor_message_recorded(
        workflow_state_path,
        &request.project_id,
        &request.workflow_id,
        &message_id,
        &turn.thread_id,
        &turn.content,
        &proposal_outcome,
    )
    .is_err()
    {
        return Ok(SupervisorResidentAnswerOutcome {
            status: "message_recorded_supervisor_incomplete".to_string(),
            reply_injected: true,
            thread_id: Some(turn.thread_id),
            supervisor_reply: None,
            message: "消息已送到主管，但主管这次没回上来——可以再发一次。".to_string(),
        });
    }
    Ok(supervisor_resident_answer_outcome(
        &proposal_outcome,
        &turn.thread_id,
        &turn.content,
    ))
}

pub(crate) struct SupervisorResidentActiveUserMessage {
    pub(crate) message_id: String,
    pub(crate) message_text: String,
}

pub(crate) fn supervisor_resident_active_user_message(
    config: &McpServerConfig,
    workflow_id: &str,
    message_id: &str,
) -> Result<SupervisorResidentActiveUserMessage, String> {
    let session = supervisor_orchestrator::load_resident_session(config)?
        .ok_or_else(|| "supervisor_resident_session_binding_missing".to_string())?;
    if session.project_id.trim().is_empty()
        || session.project_root.trim().is_empty()
        || session.workflow_id.trim().is_empty()
        || session.thread_id.trim().is_empty()
        || session.workflow_id != workflow_id
        || session.active_message_id != message_id
        || session.project_id != crate::project_id(&session.project_root)
    {
        return Err("supervisor_resident_session_binding_invalid".to_string());
    }
    let workflow_state_path = config
        .supervisor_workflow_state_path
        .as_deref()
        .ok_or_else(|| "supervisor_resident_workflow_state_path_missing".to_string())?;
    let value = crate::read_workflow_state_value(workflow_state_path)?;
    value
        .get("audit_events")
        .and_then(Value::as_array)
        .ok_or_else(|| "supervisor_resident_audit_events_missing".to_string())?
        .iter()
        .rev()
        .find_map(|event| {
            (resident_event_string(event, "event_type")
                == Some(SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT)
                && resident_event_string(event, "project_id") == Some(session.project_id.as_str())
                && resident_event_string(event, "workflow_id") == Some(workflow_id)
                && resident_event_string(event, "message_id") == Some(message_id)
                && resident_event_string(event, "actor_ref") == Some("user")
                && resident_event_string(event, "source_kind")
                    == Some("supervisor_resident_user_message"))
            .then(|| {
                resident_event_string(event, "message_text")
                    .filter(|message| !message.trim().is_empty())
                    .map(|message_text| SupervisorResidentActiveUserMessage {
                        message_id: message_id.to_string(),
                        message_text: message_text.to_string(),
                    })
            })
            .flatten()
        })
        .ok_or_else(|| "supervisor_resident_active_user_message_missing".to_string())
}

pub(crate) fn submit_supervisor_resident_message(
    workflow_state_path: &Path,
    request: &SubmitSupervisorResidentAnswerRequest,
) -> Result<SupervisorResidentAnswerOutcome, String> {
    let _guard = resident_conversation_lock()
        .lock()
        .map_err(|_| "supervisor_resident_conversation_lock_poisoned".to_string())?;
    submit_supervisor_resident_answer_with(
        &RealSupervisorResidentOneShotRunner,
        workflow_state_path,
        request,
    )
}

#[tauri::command]
pub(crate) async fn submit_supervisor_resident_answer(
    request: SubmitSupervisorResidentAnswerRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<SupervisorResidentAnswerOutcome, String> {
    let workflow_state_path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        submit_supervisor_resident_message(&workflow_state_path, &request)
    })
    .await
    .map_err(|error| format!("主管用户消息执行线程异常：{error}"))?
}

fn resident_run_id(project_root: &str) -> String {
    format!(
        "supervisor-resident:{}",
        crate::stable_id(&crate::project_id(project_root))
    )
}

fn resident_supervisor_config(workflow_state_path: &Path, run_id: &str) -> McpServerConfig {
    supervisor_config(
        workflow_state_path,
        run_id,
        SupervisorQuotaLimits {
            max_active_workers: DEFAULT_MAX_ACTIVE_WORKERS,
            max_follow_ups_per_worker: DEFAULT_MAX_FOLLOW_UPS_PER_WORKER,
            max_runtime_minutes: DEFAULT_MAX_RUNTIME_MINUTES,
        },
    )
}

fn resident_thread_invalid(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    [
        "thread not found",
        "unknown thread",
        "invalid thread",
        "session not found",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn resident_workbench_executable() -> Result<PathBuf, String> {
    #[cfg(test)]
    if let Some(executable) = std::env::var_os("SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE") {
        let executable = PathBuf::from(executable);
        if executable.is_file() {
            return Ok(executable);
        }
    }
    std::env::current_exe().map_err(|error| format!("定位工作台 MCP 可执行文件失败：{error}"))
}

fn resident_rebuild_facts(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
) -> Result<String, String> {
    let target_project_id = crate::project_id(project_root);
    let snapshot = crate::read_workflow_state_snapshot(workflow_state_path)?;
    let mut blackboard_facts = snapshot
        .project_blackboards
        .iter()
        .filter(|blackboard| {
            blackboard.project_id == target_project_id
                && (workflow_id.trim().is_empty() || blackboard.workflow_id == workflow_id)
        })
        .flat_map(|blackboard| blackboard.entries.iter())
        .filter(|entry| !entry.summary.trim().is_empty())
        .map(|entry| {
            let source_refs = entry
                .source_refs
                .iter()
                .map(|source| source.label.as_str())
                .filter(|label| !label.trim().is_empty())
                .collect::<Vec<_>>();
            let refs = if source_refs.is_empty() {
                String::new()
            } else {
                format!("（来源：{}）", source_refs.join("、"))
            };
            format!(
                "- [{}] {}：{}{}",
                entry.status, entry.title, entry.summary, refs
            )
        })
        .collect::<Vec<_>>();
    blackboard_facts.sort();
    let blackboard = if blackboard_facts.is_empty() {
        "（当前没有可注入的项目黑板既有条目。）".to_string()
    } else {
        blackboard_facts.join("\n")
    };
    let memory = crate::recall_project_memory_summary_at(workflow_state_path, project_root)
        .unwrap_or_else(|| "（当前没有活跃正式记忆。）".to_string());
    Ok(format!(
        "===== 换代/首轮核心事实（不是聊天记录）=====\n项目黑板既有条目：\n{blackboard}\n\n正式记忆 top5：\n{memory}\n\n以上事实由工作台注入；若与未核实推测冲突，以这些事实和本轮已注入材料为准。"
    ))
}

fn build_supervisor_resident_command_plan(
    project_root: &str,
    workflow_state_path: &Path,
    run_id: &str,
    generation: u64,
    workbench_executable: &Path,
    resume_thread_id: Option<&str>,
) -> Result<SupervisorCommandPlan, String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT {
        return Err(crate::legacy_product_command_blocked_message(
            "supervisor_resident_session",
        ));
    }
    let quota_limits = SupervisorQuotaLimits {
        max_active_workers: DEFAULT_MAX_ACTIVE_WORKERS,
        max_follow_ups_per_worker: DEFAULT_MAX_FOLLOW_UPS_PER_WORKER,
        max_runtime_minutes: DEFAULT_MAX_RUNTIME_MINUTES,
    };
    let mcp_args = vec![
        "__mcp_server".to_string(),
        "--role".to_string(),
        "supervisor_orchestrator".to_string(),
        "--run-id".to_string(),
        run_id.to_string(),
        "--workflow-state-path".to_string(),
        workflow_state_path.display().to_string(),
        "--max-active-workers".to_string(),
        quota_limits.max_active_workers.to_string(),
        "--max-follow-ups-per-worker".to_string(),
        quota_limits.max_follow_ups_per_worker.to_string(),
        "--max-runtime-minutes".to_string(),
        quota_limits.max_runtime_minutes.to_string(),
    ];
    let artifact_run_id = format!(
        "{run_id}:generation-{generation}:{}",
        if resume_thread_id.is_some() {
            "resume"
        } else {
            "initial"
        }
    );
    let (last_message_path, stderr_path) =
        supervisor_output_paths(workflow_state_path, &artifact_run_id)?;
    let mut argv = vec![
        "exec".to_string(),
        "--strict-config".to_string(),
        "-C".to_string(),
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
        "--sandbox".to_string(),
        "read-only".to_string(),
    ];
    if let Some(thread_id) = resume_thread_id {
        argv.push("resume".to_string());
        argv.push("--skip-git-repo-check".to_string());
        argv.push("--json".to_string());
        argv.push("--output-last-message".to_string());
        argv.push(last_message_path.display().to_string());
        argv.push("-c".to_string());
        argv.push("model_reasoning_effort=\"medium\"".to_string());
        argv.push("-c".to_string());
        argv.push("features.multi_agent=false".to_string());
        argv.push("-c".to_string());
        argv.push(format!(
            "developer_instructions={}",
            toml_string(SUPERVISOR_RESIDENT_DEVELOPER_INSTRUCTIONS)?
        ));
        argv.push(thread_id.to_string());
    } else {
        argv.push("--skip-git-repo-check".to_string());
        argv.push("--json".to_string());
        argv.push("--output-last-message".to_string());
        argv.push(last_message_path.display().to_string());
        argv.push("-c".to_string());
        argv.push("model_reasoning_effort=\"medium\"".to_string());
        argv.push("-c".to_string());
        argv.push("features.multi_agent=false".to_string());
        argv.push("-c".to_string());
        argv.push(format!(
            "developer_instructions={}",
            toml_string(SUPERVISOR_RESIDENT_DEVELOPER_INSTRUCTIONS)?
        ));
    }
    let plan = SupervisorCommandPlan {
        program: "codex".to_string(),
        argv,
        current_dir: crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
        last_message_path,
        stderr_path,
        supervisor_mcp_command: workbench_executable.to_path_buf(),
        supervisor_mcp_args: mcp_args,
    };
    validate_supervisor_resident_command_plan(&plan, project_root, resume_thread_id.is_some())?;
    Ok(plan)
}

fn validate_supervisor_resident_command_plan(
    plan: &SupervisorCommandPlan,
    project_root: &str,
    is_resume: bool,
) -> Result<(), String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT
        || plan.current_dir != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT
        || plan.program != "codex"
    {
        return Err("主管一次一发会话 cwd 与程序必须锁死固定测试项目根".to_string());
    }
    if plan.argv.first().map(String::as_str) != Some("exec")
        || plan
            .argv
            .iter()
            .filter(|arg| arg.as_str() == "--strict-config")
            .count()
            != 1
        || !argv_contains_pair(&plan.argv, "-C", crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT)
        || !argv_contains_pair(&plan.argv, "--sandbox", "read-only")
        || !plan.argv.iter().any(|arg| arg == "--json")
        || !plan.argv.iter().any(|arg| arg == "--output-last-message")
        || !plan
            .argv
            .iter()
            .any(|arg| arg == "features.multi_agent=false")
    {
        return Err("主管一次一发 argv 形状不完整，拒绝启动".to_string());
    }
    if is_resume != plan.argv.iter().any(|arg| arg == "resume") {
        return Err("主管一次一发 argv 的新建/续跑形状不一致".to_string());
    }
    if plan.argv.iter().any(|arg| {
        arg == "mcp-server"
            || arg == "--ephemeral"
            || arg == "--ignore-user-config"
            || codex_approval_bypass_arg(arg)
    }) {
        return Err("主管一次一发 argv 含常驻、临时态或审批绕过参数，已拒绝".to_string());
    }
    if !plan
        .supervisor_mcp_args
        .windows(2)
        .any(|pair| pair[0] == "--role" && pair[1] == "supervisor_orchestrator")
    {
        return Err("主管私有 CODEX_HOME 只能挂 supervisor_orchestrator".to_string());
    }
    Ok(())
}

pub(crate) fn reap_supervisor_resident_stale_sessions_at(
    workflow_state_path: &Path,
) -> Result<bool, String> {
    let config = resident_supervisor_config(
        workflow_state_path,
        &resident_run_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
    );
    reap_supervisor_resident_stale_sessions_with(
        workflow_state_path,
        &|pid| {
            let output = Command::new("/bin/kill")
                .arg("-0")
                .arg(format!("-{pid}"))
                .output()
                .map_err(|error| {
                    format!("supervisor_resident_process_group_probe_failed:{error}")
                })?;
            match output.status.code() {
                Some(0) => Ok(true),
                Some(1) => Ok(false),
                Some(code) => Err(format!(
                    "supervisor_resident_process_group_probe_exit:{code}"
                )),
                None => Err("supervisor_resident_process_group_probe_signal".to_string()),
            }
        },
        &config,
    )
}

fn reap_supervisor_resident_stale_sessions_with(
    _workflow_state_path: &Path,
    process_group_is_alive: &dyn Fn(u32) -> Result<bool, String>,
    config: &McpServerConfig,
) -> Result<bool, String> {
    let Some(session) = supervisor_orchestrator::load_resident_turn_for_reconciliation(config)?
    else {
        return Ok(false);
    };
    if !matches!(
        session.launch_status.as_str(),
        "resident_turn_starting" | "resident_turn_running" | "resident_turn_cleanup_failed"
    ) || session.host_pid == 0
    {
        return Ok(false);
    }
    match process_group_is_alive(session.host_pid) {
        Ok(true) => Ok(false),
        Ok(false) => {
            supervisor_orchestrator::record_resident_stale_turn_reaped(config, session.host_pid)
        }
        Err(error) => {
            supervisor_orchestrator::record_resident_liveness_unavailable(
                config,
                session.host_pid,
                &error,
            )?;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod resident_session_tests {
    use super::*;
    include!("supervisor_resident_oneshot_tests.rs");
}
