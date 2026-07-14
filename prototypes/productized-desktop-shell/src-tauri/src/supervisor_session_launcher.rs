// Station 2 supervisor launcher.
// Canonical contract source: docs/plans/2026-07-11-supervisor-contract-v1-draft.md.
// Any edit to that canonical contract must be synchronized into SUPERVISOR_CONTRACT_TEMPLATE.

use crate::mcp::supervisor_orchestrator::{
    self, SupervisorPilotReadModel, SupervisorPilotSessionLaunch,
};
use crate::mcp::{McpRole, McpServerConfig, SupervisorQuotaLimits};
use crate::supervisor_action_controller::{
    execute_supervisor_last_message, record_supervisor_action_quota_exceeded,
    record_supervisor_transport_failure, supervisor_action_limit, SupervisorActionResultV1,
    SupervisorActionRuntime, WorkbenchSupervisorActionAdapter,
};
use crate::supervisor_action_protocol::{
    parse_supervisor_action_proposal, SupervisorActionKind, SupervisorActionProposalV1,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

const CONTRACT_CANONICAL_SOURCE: &str = "docs/plans/2026-07-11-supervisor-contract-v1-draft.md";
const DEFAULT_REASONING_EFFORT: &str = "medium";
const ACCOUNT_DEFAULT_MODEL_ID: &str = "account_default";
const DEFAULT_MAX_ACTIVE_WORKERS: usize = 2;
const DEFAULT_MAX_FOLLOW_UPS_PER_WORKER: usize = 2;
const DEFAULT_MAX_RUNTIME_MINUTES: i64 = 30;
const SUPERVISOR_TEMP_HOME_ROOT: &str = "codex-governance-workbench-supervisor-homes";
const SUPERVISOR_TEMP_HOME_METADATA: &str = "supervisor-home.v1.json";
const SUPERVISOR_TEMP_HOME_CONFIG: &str = "config.toml";
const SUPERVISOR_TEMP_HOME_AUTH: &str = "auth.json";
const SUPERVISOR_CONTRACT_VERSION: &str = "supervisor_action_proposal.v1";

static SUPERVISOR_TEMP_HOMES_REAPED: OnceLock<Result<(), String>> = OnceLock::new();

// Keep this exact approved text synchronized with CONTRACT_CANONICAL_SOURCE.
const SUPERVISOR_CONTRACT_TEMPLATE: &str = r#"你是这单的执行主管。用户已批准方案，但执行权仍属于工作台 Syn 控制核心；你只负责判断下一步应做什么。

每次会话只能输出一个 JSON 对象，JSON 前后不得有自然语言、Markdown 或工具调用。Schema 固定为 `supervisor_action_proposal.v1`，必须含 `schema_version`、`kind`、`reason`、`expected_result`。`kind` 只能是 `dispatch_worker`、`inspect_worker`、`follow_up_worker`、`wait_worker`、`finalize`、`report_user` 或 `request_user_decision`。各动作只填写其规定目标字段。

七种动作的完整 JSON 结构如下。每次只输出其中一个对象；不得混用字段。

派发 worker：
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "dispatch_worker",
  "target": {
    "node_id": "<本单 node_id>",
    "work_item_id": "<本单 work_item_id>"
  },
  "reason": "为什么现在派发",
  "expected_result": "希望 worker 回交什么证据"
}

检查已登记 worker：
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "inspect_worker",
  "worker_id": "<已登记 worker_id>",
  "reason": "为什么现在检查回程",
  "expected_result": "获得合法结构化回程与证据"
}

追问已登记 worker：
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "follow_up_worker",
  "worker_id": "<已登记 worker_id>",
  "prompt": "请补充缺失的证据",
  "reason": "为什么需要追问",
  "expected_result": "获得补充证据或明确阻塞"
}

等待已登记 worker：
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "wait_worker",
  "worker_id": "<已登记 worker_id>",
  "reason": "worker 仍在运行",
  "expected_result": "获得最新 worker 状态"
}

提出终标建议：
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "finalize",
  "verdict": "pass",
  "reason": "合法证据已满足验收",
  "expected_result": "记录 advisory 终标建议"
}

向用户报告：
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "report_user",
  "message": "已完成的事实与证据",
  "reason": "现在需要向用户报告",
  "expected_result": "记录用户可见报告"
}

请求用户决定：
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "request_user_decision",
  "question": "需要用户确认的具体问题",
  "reason": "证据不足或存在关键方向风险",
  "expected_result": "进入等待用户决定"
}

不要输出或臆造 project_root、allowed_read、allowed_write、authorization_id、权限等级、沙箱、shell argv、可执行文件、环境变量、凭据、action_id、账本 revision、approved、bypass 或 full_access。不要调用 MCP 工具来派发、续发、终标或报告；工作台会把你唯一的动作提议绑定当前授权、任务包、配额和账本后执行，并把权威结果作为下一步输入。

主管自身始终只读。面对证据不足、越权、范围变化、不可逆风险或关键方向问题，提议 `request_user_decision`；不要假装用户已经确认、决定或取消。`finalize: pass` 只在权威 worker 回交和证据充分时提议。

状态推进规则：当上一步权威结果来自 `inspect_worker` 且 `status="completed"`、`evidence_present=true` 时，本次检查已经完成；不得对同一 worker 重复 `inspect_worker`。若证据满足主管验收，提议 `finalize`；若证据缺口可补，提议 `follow_up_worker`；若需扩权、范围变化或关键方向判断，提议 `request_user_decision`。"#;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SupervisorPilotLaunchRequest {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) authorization_id: String,
    #[serde(default)]
    pub(crate) model_id: Option<String>,
    #[serde(default = "default_reasoning_effort")]
    pub(crate) reasoning_effort: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SupervisorPilotReadModelRequest {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) run_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SupervisorPilotLaunchReceipt {
    pub(crate) run_id: String,
    pub(crate) pid: u32,
    pub(crate) opening_message: String,
    pub(crate) argv: Vec<String>,
    pub(crate) last_message_path: String,
    pub(crate) stderr_path: String,
    pub(crate) status: String,
}

#[derive(Clone, Debug)]
struct SupervisorLaunchContext {
    project_root: String,
    workflow_id: String,
    authorization_id: String,
    user_goal: String,
    user_requirement_snapshot: String,
    approved_plan_summary: String,
    worker_acceptance_criteria: Vec<String>,
    control_core_acceptance_criteria: Vec<String>,
    supervisor_acceptance_criteria: Vec<String>,
    allowed_read_roots: Vec<String>,
    allowed_write_roots: Vec<String>,
    allowed_tools: Vec<String>,
    allowed_checks: Vec<String>,
    stop_conditions: Vec<String>,
    task_package_status: String,
    pilot_task: Option<SupervisorPilotTaskReference>,
    quota_limits: SupervisorQuotaLimits,
}

#[derive(Clone, Debug)]
pub(crate) struct SupervisorPilotTaskReference {
    pub(crate) node_id: String,
    pub(crate) work_item_id: String,
    pub(crate) allowed_write: Vec<String>,
}

#[derive(Clone, Debug)]
struct SupervisorCommandPlan {
    program: String,
    argv: Vec<String>,
    current_dir: String,
    last_message_path: PathBuf,
    stderr_path: PathBuf,
    supervisor_mcp_command: PathBuf,
    supervisor_mcp_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SupervisorTemporaryHomeMetadata {
    run_id: String,
    workflow_state_path: PathBuf,
}

#[derive(Debug)]
struct SupervisorTemporaryHome {
    root: PathBuf,
    auth_path: PathBuf,
    config: McpServerConfig,
    cleanup_done: Mutex<bool>,
}

trait SupervisorProcess: Send {
    fn pid(&self) -> u32;
    fn write_opening_message(&mut self, opening_message: &str) -> Result<(), String>;
    fn wait(self: Box<Self>) -> Result<Option<i32>, String>;
    fn terminate(&mut self);
    fn temporary_home(&self) -> Option<Arc<SupervisorTemporaryHome>> {
        None
    }
}

trait SupervisorProcessSpawner {
    fn spawn(
        &self,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
    ) -> Result<Box<dyn SupervisorProcess>, String>;
}

trait SupervisorProcessRegistration: Send {
    fn unregister(self: Box<Self>);
}

trait SupervisorProcessRegistry {
    fn register(
        &self,
        workflow_state_path: &Path,
        run_id: &str,
        pid: u32,
    ) -> Box<dyn SupervisorProcessRegistration>;
}

struct CodexSupervisorProcess {
    child: Child,
    temporary_home: Arc<SupervisorTemporaryHome>,
}

impl SupervisorProcess for CodexSupervisorProcess {
    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn write_opening_message(&mut self, opening_message: &str) -> Result<(), String> {
        let mut stdin = self
            .child
            .stdin
            .take()
            .ok_or_else(|| "主管 codex stdin 不可用".to_string())?;
        stdin
            .write_all(opening_message.as_bytes())
            .map_err(|error| format!("写入主管开场白失败：{error}"))?;
        stdin
            .write_all(b"\n")
            .map_err(|error| format!("结束主管开场白输入失败：{error}"))
    }

    fn wait(mut self: Box<Self>) -> Result<Option<i32>, String> {
        let result = self
            .child
            .wait()
            .map(|status| status.code())
            .map_err(|error| format!("等待主管 codex exec 收尾失败：{error}"));
        let _ = self.temporary_home.cleanup("process_wait");
        result
    }

    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = self.temporary_home.cleanup("process_terminated");
    }

    fn temporary_home(&self) -> Option<Arc<SupervisorTemporaryHome>> {
        Some(self.temporary_home.clone())
    }
}

impl Drop for CodexSupervisorProcess {
    fn drop(&mut self) {
        let _ = self.temporary_home.cleanup("process_drop");
    }
}

struct RealSupervisorProcessSpawner;

impl SupervisorProcessSpawner for RealSupervisorProcessSpawner {
    fn spawn(
        &self,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
    ) -> Result<Box<dyn SupervisorProcess>, String> {
        let temporary_home = SupervisorTemporaryHome::create(plan, config)?;
        let output_dir = plan
            .stderr_path
            .parent()
            .ok_or_else(|| "主管 stderr 路径缺父目录，拒绝发射".to_string())?;
        fs::create_dir_all(output_dir)
            .map_err(|error| format!("创建主管 stderr 目录失败：{error}"))?;
        restrict_private_dir(output_dir)?;
        let stderr_file = fs::File::create(&plan.stderr_path)
            .map_err(|error| format!("创建主管 stderr 尸检文件失败：{error}"))?;
        restrict_private_file(&plan.stderr_path)?;
        let child = Command::new(&plan.program)
            .args(&plan.argv)
            .current_dir(&plan.current_dir)
            .env("CODEX_HOME", temporary_home.root())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|error| format!("启动主管 codex exec 失败：{error}"));
        match child {
            Ok(child) => Ok(Box::new(CodexSupervisorProcess {
                child,
                temporary_home,
            })),
            Err(error) => {
                let _ = temporary_home.cleanup("process_spawn_failed");
                Err(error)
            }
        }
    }
}

struct WorkbenchSupervisorProcessRegistration {
    registration: Option<crate::exec_process_registry::ProcessRegistration>,
}

impl SupervisorProcessRegistration for WorkbenchSupervisorProcessRegistration {
    fn unregister(mut self: Box<Self>) {
        if let Some(registration) = self.registration.take() {
            registration.unregister();
        }
    }
}

struct CleanupAwareSupervisorProcessRegistration {
    registration: Box<dyn SupervisorProcessRegistration>,
    temporary_home: Arc<SupervisorTemporaryHome>,
}

impl SupervisorProcessRegistration for CleanupAwareSupervisorProcessRegistration {
    fn unregister(self: Box<Self>) {
        let _ = self
            .temporary_home
            .cleanup("process_registration_unregistered");
        self.registration.unregister();
    }
}

struct WorkbenchSupervisorProcessRegistry;

impl SupervisorProcessRegistry for WorkbenchSupervisorProcessRegistry {
    fn register(
        &self,
        workflow_state_path: &Path,
        run_id: &str,
        pid: u32,
    ) -> Box<dyn SupervisorProcessRegistration> {
        Box::new(WorkbenchSupervisorProcessRegistration {
            registration: Some(
                crate::exec_process_registry::register_supervisor_spawned_process(
                    workflow_state_path,
                    run_id,
                    pid,
                ),
            ),
        })
    }
}

fn default_reasoning_effort() -> String {
    DEFAULT_REASONING_EFFORT.to_string()
}

#[tauri::command]
pub(crate) fn launch_supervisor_pilot(
    request: SupervisorPilotLaunchRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<SupervisorPilotLaunchReceipt, String> {
    reap_supervisor_temporary_homes_once()?;
    let context = load_authorized_launch_context(&state.workflow_state_path, &request)?;
    let executable = std::env::current_exe()
        .map_err(|error| format!("定位工作台 MCP 可执行文件失败：{error}"))?;
    let run_id = format!(
        "supervisor:{}:{}",
        crate::stable_id(&request.workflow_id),
        crate::unix_timestamp_nanos()
    );
    let config = supervisor_config(&state.workflow_state_path, &run_id, context.quota_limits);
    let opening_message = assemble_opening_message(&context);
    let command_plan = build_supervisor_command_plan(
        &request,
        &state.workflow_state_path,
        &run_id,
        &context.quota_limits,
        &executable,
    )?;
    let first_command_plan = command_plan.clone();
    let (receipt, process, registration) = spawn_supervisor_session_with(
        &RealSupervisorProcessSpawner,
        &WorkbenchSupervisorProcessRegistry,
        &state.workflow_state_path,
        &config,
        &context,
        opening_message,
        command_plan,
    )?;
    let workflow_state_path = state.workflow_state_path.clone();
    thread::spawn(move || {
        run_supervisor_action_loop(
            &RealSupervisorProcessSpawner,
            &WorkbenchSupervisorProcessRegistry,
            workflow_state_path,
            config,
            context,
            first_command_plan,
            process,
            registration,
        )
    });
    Ok(receipt)
}

#[tauri::command]
pub(crate) fn load_supervisor_pilot_read_model(
    request: SupervisorPilotReadModelRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<SupervisorPilotReadModel, String> {
    let config = McpServerConfig {
        role: McpRole::SupervisorOrchestrator,
        run_id: request.run_id,
        node_id: None,
        supervisor_workflow_state_path: Some(state.workflow_state_path.clone()),
        supervisor_quota_limits: None,
    };
    let read_model = supervisor_orchestrator::load_pilot_read_model(&config)?;
    if read_model.project_root != request.project_root
        || read_model.workflow_id != request.workflow_id
    {
        return Err("主管试点账本不属于当前项目或工作流，已拒绝读取".to_string());
    }
    Ok(read_model)
}

fn load_authorized_launch_context(
    workflow_state_path: &Path,
    request: &SupervisorPilotLaunchRequest,
) -> Result<SupervisorLaunchContext, String> {
    // 站 3b/4（2026-07-12/14 拍板）：入口先按 mario 根等值放行到「读授权段」，具体是 3b 零写根
    // 还是 4 单一同根写根，必须由下方 ensure_supervisor_pilot_write_scope 拿到授权段后全判。
    if !crate::workflow_engine_test_project_unsealed(&request.project_root)
        && !crate::station3b_readonly_project_root(&request.project_root)
    {
        return Err(crate::legacy_product_command_blocked_message(
            "launch_supervisor_pilot",
        ));
    }
    if request.reasoning_effort.trim().is_empty() {
        return Err("主管试点必须显式提供 reasoning_effort".to_string());
    }
    let project_id = crate::project_id(&request.project_root);
    let authorization_store = crate::plan_authorization_store::load_store(
        workflow_state_path,
        crate::unix_timestamp_ms(),
    )?;
    let authorization = authorization_store
        .authorizations
        .iter()
        .find(|authorization| {
            authorization.project_id == project_id
                && authorization.workflow_id == request.workflow_id
                && authorization.authorization_id == request.authorization_id
                && authorization.status == crate::PlanAuthorizationStatus::Active
        })
        .ok_or_else(|| "当前单不存在指定的 active 授权段，不能发射主管试点".to_string())?;
    if authorization.user_confirmation.is_none() {
        return Err("当前授权段缺用户确认，不能发射主管试点".to_string());
    }
    if authorization
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms < crate::unix_timestamp_ms())
    {
        return Err("当前授权段已经过期，不能发射主管试点".to_string());
    }
    ensure_supervisor_pilot_write_scope(
        &request.project_root,
        &authorization.scope.allowed_write_roots,
    )?;
    let proposal_id = authorization
        .source_proposal_id
        .as_deref()
        .ok_or_else(|| "当前授权段缺 source proposal，不能组装主管上下文".to_string())?;
    let proposal_store = crate::project_consultation_proposal_store::load_store(
        workflow_state_path,
        crate::unix_timestamp_ms(),
    )?;
    let proposal = proposal_store
        .proposals
        .iter()
        .find(|proposal| {
            proposal.proposal_id == proposal_id
                && proposal.project_id == project_id
                && proposal.workflow_id == request.workflow_id
        })
        .ok_or_else(|| "当前 active 授权段找不到同单方案，不能组装主管上下文".to_string())?;
    let quota_limits = SupervisorQuotaLimits {
        max_active_workers: authorization
            .scope
            .max_worker_dispatches
            .and_then(|value| usize::try_from(value).ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_ACTIVE_WORKERS),
        max_follow_ups_per_worker: DEFAULT_MAX_FOLLOW_UPS_PER_WORKER,
        max_runtime_minutes: authorization
            .scope
            .max_runtime_minutes
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_RUNTIME_MINUTES),
    };
    let user_requirement_snapshot = proposal_user_requirement_snapshot(proposal).to_string();
    let worker_acceptance_criteria = proposal.worker_acceptance_criteria.clone();
    let control_core_acceptance_criteria = proposal.control_core_acceptance_criteria.clone();
    let supervisor_acceptance_criteria = proposal.supervisor_acceptance_criteria.clone();
    // 站 3b/4：授权段写根原样进入任务包+prepared dispatch。3b 空写根 → worker read-only；4 的唯一
    // mario 根 → worker workspace-write。主管自身仍只读，真实项目目录只对受控 worker 开放。
    let pilot_task = Some(prepare_supervisor_pilot_write_task(
        workflow_state_path,
        request,
        proposal,
        authorization,
        authorization_store.revision,
    )?);
    Ok(SupervisorLaunchContext {
        project_root: request.project_root.clone(),
        workflow_id: request.workflow_id.clone(),
        authorization_id: authorization.authorization_id.clone(),
        user_goal: proposal.user_goal.clone(),
        user_requirement_snapshot,
        approved_plan_summary: proposal.goal_summary.clone(),
        worker_acceptance_criteria,
        control_core_acceptance_criteria,
        supervisor_acceptance_criteria,
        allowed_read_roots: authorization.scope.allowed_read_roots.clone(),
        allowed_write_roots: authorization.scope.allowed_write_roots.clone(),
        allowed_tools: authorization.scope.allowed_tools.clone(),
        allowed_checks: authorization.scope.allowed_checks.clone(),
        stop_conditions: authorization
            .scope
            .stop_conditions
            .iter()
            .map(|condition| condition.summary.clone())
            .collect(),
        task_package_status: task_package_status_summary(
            workflow_state_path,
            &request.workflow_id,
        )?,
        pilot_task,
        quota_limits,
    })
}

fn ensure_supervisor_pilot_write_scope(
    project_root: &str,
    allowed_write_roots: &[String],
) -> Result<(), String> {
    // mario 项目：只允许 3b 的零写根或 4 的唯一精确同根写根；同一目录但两种语义都必须完整判形。
    if crate::station3b_readonly_project_root(project_root) {
        if crate::station3b_readonly_project_unsealed(project_root, allowed_write_roots)
            || crate::station4_write_project_unsealed(project_root, allowed_write_roots)
        {
            return Ok(());
        }
        return Err(
            "mario test 项目只允许站 3b 的零写根或站 4 的唯一精确写根；当前授权段写根形态不匹配，已拒绝发射"
                .to_string(),
        );
    }
    // 函数自身 fail-closed：非测试非 mario 项目即使零写根也拒——不再只靠入口闸兜底。
    if crate::workflow_engine_test_project_unsealed(project_root)
        && allowed_write_roots
            .iter()
            .all(|root| root == crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT)
    {
        return Ok(());
    }
    Err("主管编排写入试点只允许固定测试项目根；当前授权段含越界写根".to_string())
}

fn prepare_supervisor_pilot_write_task(
    workflow_state_path: &Path,
    request: &SupervisorPilotLaunchRequest,
    proposal: &crate::ProjectConsultationProposal,
    authorization: &crate::PlanAuthorization,
    authorization_revision: i64,
) -> Result<SupervisorPilotTaskReference, String> {
    let app_state = crate::AppState::new();
    let index = crate::read_index(&app_state)?;
    prepare_supervisor_pilot_write_task_for_index(
        workflow_state_path,
        &index,
        request,
        proposal,
        authorization,
        authorization_revision,
    )
}

// 主管试点真实物化路径的可注入入口。生产调用仍从 AppState 读取 index；测试只替换索引，不能绕开
// proposal → authorization → planned task → task package 的同一条路径。
pub(crate) fn prepare_supervisor_pilot_write_task_for_index(
    workflow_state_path: &Path,
    index: &Value,
    request: &SupervisorPilotLaunchRequest,
    proposal: &crate::ProjectConsultationProposal,
    authorization: &crate::PlanAuthorization,
    authorization_revision: i64,
) -> Result<SupervisorPilotTaskReference, String> {
    let planned_task_id = supervisor_pilot_planned_task_id(&authorization.authorization_id);
    validate_supervisor_pilot_role_criteria(
        &proposal.worker_acceptance_criteria,
        &proposal.control_core_acceptance_criteria,
        &proposal.supervisor_acceptance_criteria,
    )?;
    let worker_acceptance_criteria = proposal.worker_acceptance_criteria.clone();
    let task = crate::ProjectDirectorPlannedTask {
        planned_task_id: planned_task_id.clone(),
        title: supervisor_pilot_worker_task_title(&worker_acceptance_criteria),
        task_goal: supervisor_pilot_worker_objective(&worker_acceptance_criteria),
        scope: crate::ProjectDirectorTaskScope {
            project_id: authorization.project_id.clone(),
            workflow_id: request.workflow_id.clone(),
            target_role: "codex-dev".to_string(),
            task_package_kind: "task_package".to_string(),
            allowed_read_scope: authorization.scope.allowed_read_roots.clone(),
            allowed_write_scope: authorization.scope.allowed_write_roots.clone(),
            available_skills: vec![],
            available_knowledge_refs: vec![],
            callable_tool_capabilities: authorization.scope.allowed_tools.clone(),
            // 授权段的 allowed_checks 可能含控制核心或主管的验收；它们不能作为 worker 任务包内容。
            required_checks: vec![],
            stop_conditions: authorization
                .scope
                .stop_conditions
                .iter()
                .map(|condition| condition.summary.clone())
                .collect(),
            timeout_policy: None,
            failure_policy: Some("失败即停并向主管报告，不重试、不扩权。".to_string()),
            forbidden_actions: vec![
                "不读写 /Users/yoyi/.codex。".to_string(),
                "不越过本任务包 allowed_write。".to_string(),
                "不安装依赖、不访问网络、不修改任务目标外文件。".to_string(),
            ],
            model_id: None,
        },
        depends_on: vec![],
        worker_acceptance_criteria: worker_acceptance_criteria.clone(),
        control_core_acceptance_criteria: proposal.control_core_acceptance_criteria.clone(),
        supervisor_acceptance_criteria: proposal.supervisor_acceptance_criteria.clone(),
        acceptance_criteria: worker_acceptance_criteria,
        report_format: vec![
            "status: done|partial|failed|blocked".to_string(),
            "changed_what: 实际改动文件".to_string(),
            "evidence: 回读或检查证据".to_string(),
        ],
        status: "planned".to_string(),
        guard_result: None,
        work_item_id: None,
        workflow_node_id: None,
        task_package_id: None,
        memory_packet_snapshot_id: None,
        prepared_dispatch_id: None,
        blocked_reasons: vec![],
    };
    let prepared = crate::prepare_authorized_auto_dispatch_for_index_at(
        workflow_state_path,
        &index,
        &crate::PrepareAuthorizedAutoDispatchInput {
            project_root: request.project_root.clone(),
            project_id: authorization.project_id.clone(),
            workflow_id: request.workflow_id.clone(),
            proposal_id: proposal.proposal_id.clone(),
            authorization_id: authorization.authorization_id.clone(),
            actor_id: "supervisor_orchestrator".to_string(),
            planned_tasks: vec![task],
            expected_workflow_revision: None,
            expected_authorization_revision: Some(authorization_revision),
            chain_binds_per_task: true,
            force_fresh_task_session: true,
        },
    )?;
    let task = prepared
        .plan
        .planned_tasks
        .into_iter()
        .find(|task| task.planned_task_id == planned_task_id)
        .ok_or_else(|| "主管任务包物化后未返回对应任务，已拒绝发射".to_string())?;
    if task.status != "prepared" {
        return Err(format!(
            "主管任务包尚不可派发（状态 {}）：{}",
            task.status,
            task.blocked_reasons.join("；")
        ));
    }
    let task_package_id = task
        .task_package_id
        .as_deref()
        .ok_or_else(|| "主管任务包物化后缺 task package id，已拒绝发射".to_string())?;
    persist_supervisor_pilot_user_requirement_snapshot(
        workflow_state_path,
        task_package_id,
        proposal_user_requirement_snapshot(proposal),
    )?;
    Ok(SupervisorPilotTaskReference {
        node_id: task
            .workflow_node_id
            .ok_or_else(|| "主管任务包缺 worker node，已拒绝发射".to_string())?,
        work_item_id: task
            .work_item_id
            .ok_or_else(|| "主管任务包缺 work item，已拒绝发射".to_string())?,
        allowed_write: authorization.scope.allowed_write_roots.clone(),
    })
}

fn validate_supervisor_pilot_role_criteria(
    worker_acceptance_criteria: &[String],
    control_core_acceptance_criteria: &[String],
    supervisor_acceptance_criteria: &[String],
) -> Result<(), String> {
    for (field, criteria) in [
        ("worker_acceptance_criteria", worker_acceptance_criteria),
        (
            "control_core_acceptance_criteria",
            control_core_acceptance_criteria,
        ),
        (
            "supervisor_acceptance_criteria",
            supervisor_acceptance_criteria,
        ),
    ] {
        if criteria.is_empty() || criteria.iter().any(|criterion| criterion.trim().is_empty()) {
            return Err(format!(
                "主管任务包缺有效 {field}；拒绝从旧版统一验收字段猜测职责"
            ));
        }
    }
    Ok(())
}

fn supervisor_pilot_worker_objective(acceptance_criteria: &[String]) -> String {
    let acceptance_criteria = acceptance_criteria
        .iter()
        .enumerate()
        .map(|(index, criterion)| format!("{}. {criterion}", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    format!("执行下列 worker 验收：\n{}", acceptance_criteria)
}

fn supervisor_pilot_worker_task_title(acceptance_criteria: &[String]) -> String {
    acceptance_criteria
        .first()
        .map(|criterion| format!("Worker：{criterion}"))
        .unwrap_or_else(|| "Worker 任务".to_string())
}

fn proposal_user_requirement_snapshot(proposal: &crate::ProjectConsultationProposal) -> &str {
    if proposal.user_requirement_snapshot.is_empty() {
        &proposal.user_goal
    } else {
        &proposal.user_requirement_snapshot
    }
}

fn persist_supervisor_pilot_user_requirement_snapshot(
    workflow_state_path: &Path,
    task_package_id: &str,
    user_requirement_snapshot: &str,
) -> Result<(), String> {
    let timestamp = crate::unix_timestamp_string();
    let mut value = crate::read_workflow_state_value(workflow_state_path)?;
    let artifact = value
        .get_mut("artifacts")
        .and_then(Value::as_array_mut)
        .and_then(|artifacts| {
            artifacts.iter_mut().find(|artifact| {
                crate::optional_string_from(artifact, "artifact_id").as_deref()
                    == Some(task_package_id)
            })
        })
        .ok_or_else(|| "主管任务包物化后找不到 task package artifact，已拒绝发射".to_string())?;
    artifact["user_requirement_snapshot"] = Value::String(user_requirement_snapshot.to_string());
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "supervisor-pilot-user-requirement-snapshot",
        task_package_id,
        &timestamp,
    );
    crate::array_mut(&mut value, "audit_events")?.push(json!({
        "event_id": audit_event_id,
        "event_type": "supervisor_pilot_user_requirement_snapshot_recorded",
        "target_ref": task_package_id,
        "actor_ref": "supervisor_orchestrator",
        "source_kind": "workspace_state",
        "permission_level": "authorized_supervisor_execution",
        "created_at": timestamp.clone(),
        "reason": "主管试点已将用户原始需求快照回填任务包 artifact；审计不记录需求正文。"
    }));
    value["updated_at"] = Value::String(timestamp);
    crate::write_m5b_batch2_workflow_state(
        workflow_state_path,
        "supervisor_pilot_user_requirement_snapshot_recorded",
        &value,
    )
}


fn supervisor_pilot_planned_task_id(authorization_id: &str) -> String {
    let authorization_hash = crate::utils::hash::sha256_hex(authorization_id);
    format!(
        "planned-task:supervisor-pilot:{}",
        &authorization_hash[..24]
    )
}

fn task_package_status_summary(
    workflow_state_path: &Path,
    workflow_id: &str,
) -> Result<String, String> {
    let value = crate::read_workflow_state_value(workflow_state_path)?;
    let mut work_item_states = BTreeMap::<String, usize>::new();
    if let Some(work_items) = value.get("work_items").and_then(Value::as_array) {
        for item in work_items.iter().filter(|item| {
            crate::optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id)
        }) {
            let state = crate::optional_string_from(item, "state")
                .or_else(|| crate::optional_string_from(item, "status"))
                .unwrap_or_else(|| "unknown".to_string());
            *work_item_states.entry(state).or_default() += 1;
        }
    }
    let task_package_count = value
        .get("artifacts")
        .and_then(Value::as_array)
        .map(|artifacts| {
            artifacts
                .iter()
                .filter(|artifact| {
                    crate::optional_string_from(artifact, "workflow_id").as_deref()
                        == Some(workflow_id)
                        && ["artifact_type", "kind", "type"].iter().any(|key| {
                            crate::optional_string_from(artifact, key)
                                .is_some_and(|value| value == "task_package")
                        })
                })
                .count()
        })
        .unwrap_or(0);
    let states = if work_item_states.is_empty() {
        "无工作项记录".to_string()
    } else {
        work_item_states
            .into_iter()
            .map(|(state, count)| format!("{state}={count}"))
            .collect::<Vec<_>>()
            .join("，")
    };
    Ok(format!(
        "已物化任务包 {task_package_count} 份；工作项状态：{states}。"
    ))
}

fn assemble_opening_message(context: &SupervisorLaunchContext) -> String {
    let pilot_task = context
        .pilot_task
        .as_ref()
        .map(|task| {
            format!(
                "node_id={}；work_item_id={}；allowed_write={}",
                task.node_id,
                task.work_item_id,
                join_or_none(&task.allowed_write)
            )
        })
        .unwrap_or_else(|| "无（只读单可直接调查，也可按需派只读 worker）".to_string());
    let worker_acceptance = supervisor_acceptance_list(&context.worker_acceptance_criteria);
    let control_core_acceptance = supervisor_acceptance_list(&context.control_core_acceptance_criteria);
    let supervisor_acceptance = supervisor_acceptance_list(&context.supervisor_acceptance_criteria);
    format!(
        "{SUPERVISOR_CONTRACT_TEMPLATE}\n\n===== 本单上下文（已批准，来自工作台正本）=====\n契约正本：{CONTRACT_CANONICAL_SOURCE}\n用户原始需求快照（逐字保留）：\n{}\n\n用户目标（方案提炼）：{}\n已批方案摘要：{}\nWorker 验收（只由 worker 完成）：\n{}\n控制核心验收（不得下放给 worker）：\n{}\n主管验收（不得下放给 worker）：\n{}\n授权范围：\n- 授权段：{}\n- 项目根：{}\n- 可读根：{}\n- 可写根：{}\n- 可用工具：{}\n- 可用检查：{}\n- 停止条件：{}\n- 主管配额：并发 {}，每 worker 追问 {}，总时长 {} 分钟\n任务包现状：{}\n本单可派任务：{}\n\n现在只输出第一个 `SupervisorActionProposalV1` JSON。若要派 worker，必须使用上方完整嵌套 `target` 结构；Syn 会从正本派生 allowed_write，不能由你填写。\n",
        context.user_requirement_snapshot,
        context.user_goal,
        context.approved_plan_summary,
        worker_acceptance,
        control_core_acceptance,
        supervisor_acceptance,
        context.authorization_id,
        context.project_root,
        join_or_none(&context.allowed_read_roots),
        join_or_none(&context.allowed_write_roots),
        join_or_none(&context.allowed_tools),
        join_or_none(&context.allowed_checks),
        join_or_none(&context.stop_conditions),
        context.quota_limits.max_active_workers,
        context.quota_limits.max_follow_ups_per_worker,
        context.quota_limits.max_runtime_minutes,
        context.task_package_status,
        pilot_task,
    )
}

fn supervisor_acceptance_list(criteria: &[String]) -> String {
    if criteria.is_empty() {
        return "- 无".to_string();
    }
    criteria
        .iter()
        .map(|criterion| format!("- {criterion}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn proposal_example_for_kind(context: &SupervisorLaunchContext, kind: Option<&str>) -> String {
    let (node_id, work_item_id) = context
        .pilot_task
        .as_ref()
        .map(|task| (task.node_id.as_str(), task.work_item_id.as_str()))
        .unwrap_or(("<本单 node_id>", "<本单 work_item_id>"));
    let example = match kind {
        Some("dispatch_worker") => json!({
            "schema_version": "supervisor_action_proposal.v1",
            "kind": "dispatch_worker",
            "target": {"node_id": node_id, "work_item_id": work_item_id},
            "reason": "为什么现在派发",
            "expected_result": "希望 worker 回交什么证据",
        }),
        Some("inspect_worker") => json!({
            "schema_version": "supervisor_action_proposal.v1",
            "kind": "inspect_worker",
            "worker_id": "<已登记 worker_id>",
            "reason": "为什么现在检查回程",
            "expected_result": "获得合法结构化回程与证据",
        }),
        Some("follow_up_worker") => json!({
            "schema_version": "supervisor_action_proposal.v1",
            "kind": "follow_up_worker",
            "worker_id": "<已登记 worker_id>",
            "prompt": "请补充缺失的证据",
            "reason": "为什么需要追问",
            "expected_result": "获得补充证据或明确阻塞",
        }),
        Some("wait_worker") => json!({
            "schema_version": "supervisor_action_proposal.v1",
            "kind": "wait_worker",
            "worker_id": "<已登记 worker_id>",
            "reason": "worker 仍在运行",
            "expected_result": "获得最新 worker 状态",
        }),
        Some("finalize") => json!({
            "schema_version": "supervisor_action_proposal.v1",
            "kind": "finalize",
            "verdict": "pass",
            "reason": "合法证据已满足验收",
            "expected_result": "记录 advisory 终标建议",
        }),
        Some("report_user") => json!({
            "schema_version": "supervisor_action_proposal.v1",
            "kind": "report_user",
            "message": "已完成的事实与证据",
            "reason": "现在需要向用户报告",
            "expected_result": "记录用户可见报告",
        }),
        Some("request_user_decision") => json!({
            "schema_version": "supervisor_action_proposal.v1",
            "kind": "request_user_decision",
            "question": "需要用户确认的具体问题",
            "reason": "证据不足或存在关键方向风险",
            "expected_result": "进入等待用户决定",
        }),
        _ => {
            return "（原消息没有可识别 kind；请使用上方七种完整结构中的一种。）".to_string();
        }
    };
    serde_json::to_string_pretty(&example).expect("supervisor proposal example is serializable")
}

fn intended_action_kind(raw_message: &str) -> Option<String> {
    serde_json::from_str::<Value>(raw_message)
        .ok()?
        .get("kind")?
        .as_str()
        .filter(|kind| {
            matches!(
                *kind,
                "dispatch_worker"
                    | "inspect_worker"
                    | "follow_up_worker"
                    | "wait_worker"
                    | "finalize"
                    | "report_user"
                    | "request_user_decision"
            )
        })
        .map(str::to_string)
}

fn assemble_protocol_correction_message(
    context: &SupervisorLaunchContext,
    protocol_error: &str,
    raw_message: &str,
) -> String {
    let intended_kind = intended_action_kind(raw_message);
    let action_note = intended_kind
        .as_deref()
        .map(|kind| format!("原动作 kind 为 `{kind}`；下方只给出该动作的正确结构。"))
        .unwrap_or_else(|| {
            "无法识别原动作 kind；请从上方七种完整结构中选择实际要提议的动作。".to_string()
        });
    format!(
        "{SUPERVISOR_CONTRACT_TEMPLATE}\n\n===== 主管输出格式错误 =====\n当前无效动作未执行：Syn 没有为这条无效动作调用 adapter 或启动 worker。具体错误：{protocol_error}\n{action_note}\n\n这是唯一一次格式纠正机会。现在只能输出一个严格合法的 `SupervisorActionProposalV1` JSON 对象；不要附加自然语言或 Markdown。\n\n===== 正确 JSON 示例 =====\n{}\n\n本单绑定：dispatch_worker 的 node/work item 必须使用本单正本值。请直接输出纠正后的 JSON。",
        proposal_example_for_kind(context, intended_kind.as_deref()),
    )
}

fn assemble_next_step_message(
    context: &SupervisorLaunchContext,
    previous_proposal: Option<&SupervisorActionProposalV1>,
    result: &SupervisorActionResultV1,
) -> String {
    let result_json = serde_json::to_string_pretty(result)
        .unwrap_or_else(|_| "{\"status\":\"transport_failed\"}".to_string());
    let progress_instruction = match previous_proposal.map(|proposal| &proposal.action) {
        Some(SupervisorActionKind::InspectWorker { worker_id })
            if result.status == "completed" && result.evidence_present =>
        {
            format!(
                "同一 worker `{worker_id}` 的 inspect_worker 已完成且证据已落账。禁止再次输出同一 inspect_worker；若证据满足主管验收，下一步提议 finalize；若证据缺口可补，提议 follow_up_worker；若需要扩权、范围变化或关键方向判断，提议 request_user_decision。"
            )
        }
        Some(SupervisorActionKind::Finalize { .. }) if result.status == "completed" => {
            "终标建议已经落账。禁止重复 finalize；下一步应提议 report_user。".to_string()
        }
        _ => "上一步动作已由 Syn 处理；不要把已完成动作当成未执行，请依据权威结果推进到新的动作。".to_string(),
    };
    format!(
        "{SUPERVISOR_CONTRACT_TEMPLATE}\n\n===== 当前本单绑定 =====\n用户原始需求快照（逐字保留）：\n{}\n\n用户目标（方案提炼）：{}\n已批方案摘要：{}\nWorker 验收（只由 worker 完成）：\n{}\n控制核心验收（不得下放给 worker）：\n{}\n主管验收（不得下放给 worker）：\n{}\n任务包现状：{}\n\n===== 上一步权威执行结果（由 Syn 写入，不能改写）=====\n{result_json}\n\n===== Syn 状态推进约束 =====\n{progress_instruction}\n\n基于这条权威结果，只输出下一个 `SupervisorActionProposalV1` JSON。",
        context.user_requirement_snapshot,
        context.user_goal,
        context.approved_plan_summary,
        supervisor_acceptance_list(&context.worker_acceptance_criteria),
        supervisor_acceptance_list(&context.control_core_acceptance_criteria),
        supervisor_acceptance_list(&context.supervisor_acceptance_criteria),
        context.task_package_status
    )
}

fn build_supervisor_command_plan(
    request: &SupervisorPilotLaunchRequest,
    workflow_state_path: &Path,
    run_id: &str,
    quota_limits: &SupervisorQuotaLimits,
    workbench_executable: &Path,
) -> Result<SupervisorCommandPlan, String> {
    // 站 3b/4 按 mario 根等值放行（写根形状已在 load_authorized_launch_context 全判）。主管会话 cwd
    // 与 -C 仍死锁固定测试项目根：主管只读投影不进 mario 项目目录，进真实项目的只有受控 worker。
    if !crate::workflow_engine_test_project_unsealed(&request.project_root)
        && !crate::station3b_readonly_project_root(&request.project_root)
    {
        return Err(crate::legacy_product_command_blocked_message(
            "launch_supervisor_pilot",
        ));
    }
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
    let effort = toml_string(&request.reasoning_effort)?;
    let (last_message_path, stderr_path) = supervisor_output_paths(workflow_state_path, run_id)?;
    let mut argv = vec![
        "exec".to_string(),
        "--ephemeral".to_string(),
        "-C".to_string(),
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
        "--sandbox".to_string(),
        "read-only".to_string(),
        "--skip-git-repo-check".to_string(),
        "--json".to_string(),
        "--output-last-message".to_string(),
        last_message_path.display().to_string(),
        "-c".to_string(),
        format!("model_reasoning_effort={effort}"),
        "-c".to_string(),
        "features.multi_agent=false".to_string(),
    ];
    if let Some(model_id) = explicit_model_id(request) {
        argv.extend(["--model".to_string(), model_id.to_string()]);
    }
    validate_supervisor_argv(&argv, &request.project_root)?;
    Ok(SupervisorCommandPlan {
        program: "codex".to_string(),
        argv,
        current_dir: crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
        last_message_path,
        stderr_path,
        supervisor_mcp_command: workbench_executable.to_path_buf(),
        supervisor_mcp_args: mcp_args,
    })
}

fn explicit_model_id(request: &SupervisorPilotLaunchRequest) -> Option<&str> {
    request
        .model_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn supervisor_output_paths(
    workflow_state_path: &Path,
    run_id: &str,
) -> Result<(PathBuf, PathBuf), String> {
    let parent = crate::utils::store_paths::runtime_artifact_dir(
        workflow_state_path,
        "supervisor",
        run_id,
    )?;
    Ok((
        parent.join("step-0.last-message.txt"),
        parent.join("step-0.stderr.txt"),
    ))
}

fn command_plan_for_next_step(
    previous: &SupervisorCommandPlan,
    _run_id: &str,
    step: usize,
) -> Result<SupervisorCommandPlan, String> {
    let parent = previous
        .last_message_path
        .parent()
        .ok_or_else(|| "主管输出材料缺父目录".to_string())?;
    let last_message_path = parent.join(format!("step-{step}.last-message.txt"));
    let stderr_path = parent.join(format!("step-{step}.stderr.txt"));
    let mut plan = previous.clone();
    plan.last_message_path = last_message_path.clone();
    plan.stderr_path = stderr_path;
    for index in 0..plan.argv.len().saturating_sub(1) {
        if plan.argv[index] == "--output-last-message" {
            plan.argv[index + 1] = last_message_path.display().to_string();
            return Ok(plan);
        }
    }
    Err("主管 argv 缺 --output-last-message，拒绝启动下一步。".to_string())
}

fn validate_supervisor_argv(argv: &[String], project_root: &str) -> Result<(), String> {
    if !crate::workflow_engine_test_project_unsealed(project_root)
        && !crate::station3b_readonly_project_root(project_root)
    {
        return Err("主管试点只能锁定固定测试项目或 mario test 项目".to_string());
    }
    // 3b/4 都不例外：主管会话 cwd 恒为固定测试项目根，真实项目目录只对受控 worker 开放。
    if !argv_contains_pair(argv, "-C", crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT) {
        return Err("主管会话 -C 必须留在固定测试项目根，拒绝启动".to_string());
    }
    if !argv_contains_pair(argv, "--sandbox", "read-only") {
        return Err("主管试点必须使用 read-only 沙箱".to_string());
    }
    if argv
        .iter()
        .any(|argument| argument == "--ignore-user-config")
    {
        return Err("主管试点必须改用临时 CODEX_HOME，不能清空主管 MCP 配置".to_string());
    }
    if !argv
        .iter()
        .any(|argument| argument == "features.multi_agent=false")
    {
        return Err("主管试点必须关闭原生多代理功能".to_string());
    }
    if argv
        .iter()
        .any(|argument| codex_approval_bypass_arg(argument))
    {
        return Err("主管试点 argv 含审批绕过参数，已拒绝".to_string());
    }
    Ok(())
}

// Keep parity with the existing codex_approval_bypass_arg deny list in manual_relay.rs.
// That module is a Station 2 hard no-touch boundary, so this launcher owns the same predicate locally.
fn codex_approval_bypass_arg(argument: &str) -> bool {
    argument == "--full-auto"
        || argument.contains("dangerously-bypass")
        || argument.starts_with("--approval")
        || argument == "full-auto"
}

fn argv_contains_pair(argv: &[String], flag: &str, value: &str) -> bool {
    argv.windows(2).any(|pair| {
        pair.first().is_some_and(|argument| argument == flag)
            && pair.get(1).is_some_and(|argument| argument == value)
    })
}

fn toml_string(value: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("编码一次性 codex 配置失败：{error}"))
}

impl SupervisorTemporaryHome {
    fn create(plan: &SupervisorCommandPlan, config: &McpServerConfig) -> Result<Arc<Self>, String> {
        let auth_source = default_codex_auth_path()?;
        Self::create_at(
            &supervisor_temporary_homes_root(),
            &auth_source,
            plan,
            config,
        )
    }

    fn create_at(
        base: &Path,
        auth_source: &Path,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
    ) -> Result<Arc<Self>, String> {
        let auth_metadata = fs::metadata(auth_source).map_err(|error| {
            format!(
                "主管试点需要读取 ~/.codex/auth.json，但当前不可用 {}：{error}",
                auth_source.display()
            )
        })?;
        if !auth_metadata.is_file() {
            return Err("主管试点拒绝使用非普通文件的 ~/.codex/auth.json".to_string());
        }
        let root = create_private_supervisor_home_dir(base, &config.run_id)?;
        let setup = (|| -> Result<(), String> {
            let config_toml = supervisor_mcp_config_toml(plan)?;
            write_private_file(
                &root.join(SUPERVISOR_TEMP_HOME_CONFIG),
                config_toml.as_bytes(),
            )?;
            let metadata = SupervisorTemporaryHomeMetadata {
                run_id: config.run_id.clone(),
                workflow_state_path: config
                    .supervisor_workflow_state_path
                    .clone()
                    .ok_or_else(|| "主管临时 home 缺 workflow state 路径".to_string())?,
            };
            let metadata = serde_json::to_vec(&metadata)
                .map_err(|error| format!("序列化主管临时 home 元数据失败：{error}"))?;
            write_private_file(&root.join(SUPERVISOR_TEMP_HOME_METADATA), &metadata)?;
            create_auth_symlink(auth_source, &root.join(SUPERVISOR_TEMP_HOME_AUTH))
        })();
        if let Err(error) = setup {
            let _ = fs::remove_dir_all(&root);
            return Err(error);
        }
        let temporary_home = Arc::new(Self {
            auth_path: root.join(SUPERVISOR_TEMP_HOME_AUTH),
            root,
            config: config.clone(),
            cleanup_done: Mutex::new(false),
        });
        if let Err(error) = supervisor_orchestrator::record_pilot_temporary_home_created(
            &temporary_home.config,
            temporary_home.root(),
        ) {
            let _ = temporary_home.cleanup("temporary_home_create_audit_failed");
            return Err(format!("主管临时 home 已创建但账本登记失败：{error}"));
        }
        Ok(temporary_home)
    }

    fn from_orphan(root: PathBuf, metadata: SupervisorTemporaryHomeMetadata) -> Self {
        Self {
            auth_path: root.join(SUPERVISOR_TEMP_HOME_AUTH),
            root,
            config: McpServerConfig {
                role: McpRole::SupervisorOrchestrator,
                run_id: metadata.run_id,
                node_id: None,
                supervisor_workflow_state_path: Some(metadata.workflow_state_path),
                supervisor_quota_limits: None,
            },
            cleanup_done: Mutex::new(false),
        }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn cleanup(&self, cleanup_trigger: &str) -> Result<(), String> {
        let mut cleanup_done = self
            .cleanup_done
            .lock()
            .map_err(|_| "主管临时 home 清理锁异常".to_string())?;
        if *cleanup_done {
            return Ok(());
        }
        let token_was_refreshed = matches!(
            fs::symlink_metadata(&self.auth_path),
            Ok(metadata) if !metadata.file_type().is_symlink()
        );
        if token_was_refreshed {
            let _ = fs::remove_file(&self.auth_path);
        }
        let cleanup_result = fs::remove_dir_all(&self.root).map_err(|error| {
            format!(
                "清理主管临时 CODEX_HOME 失败 {}：{error}",
                self.root.display()
            )
        });
        let cleanup_succeeded = cleanup_result.is_ok();
        let audit_result = supervisor_orchestrator::record_pilot_temporary_home_cleaned(
            &self.config,
            &self.root,
            cleanup_trigger,
            token_was_refreshed,
            cleanup_succeeded,
        );
        if cleanup_succeeded {
            *cleanup_done = true;
        }
        cleanup_result?;
        audit_result.map_err(|error| format!("主管临时 home 清理账本登记失败：{error}"))
    }
}

impl Drop for SupervisorTemporaryHome {
    fn drop(&mut self) {
        let _ = self.cleanup("temporary_home_drop");
    }
}

fn supervisor_temporary_homes_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/private/tmp").join(SUPERVISOR_TEMP_HOME_ROOT)
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::temp_dir().join(SUPERVISOR_TEMP_HOME_ROOT)
    }
}

fn default_codex_auth_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "无法定位 HOME，不能创建主管认证传送门".to_string())?;
    Ok(home.join(".codex").join(SUPERVISOR_TEMP_HOME_AUTH))
}

fn create_private_supervisor_home_dir(base: &Path, run_id: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(base)
        .map_err(|error| format!("创建主管临时 home 根目录失败 {}：{error}", base.display()))?;
    restrict_private_dir(base)?;
    for attempt in 0..=3 {
        let root = base.join(format!(
            "run-{}-{}-{attempt}",
            crate::stable_id(run_id),
            crate::unix_timestamp_nanos()
        ));
        match fs::create_dir(&root) {
            Ok(()) => {
                restrict_private_dir(&root)?;
                return Ok(root);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "创建主管临时 home 失败 {}：{error}",
                    root.display()
                ));
            }
        }
    }
    Err("无法分配唯一主管临时 CODEX_HOME".to_string())
}

fn supervisor_mcp_config_toml(plan: &SupervisorCommandPlan) -> Result<String, String> {
    let mut server = toml::map::Map::new();
    server.insert(
        "command".to_string(),
        toml::Value::String(plan.supervisor_mcp_command.display().to_string()),
    );
    server.insert(
        "args".to_string(),
        toml::Value::Array(
            plan.supervisor_mcp_args
                .iter()
                .cloned()
                .map(toml::Value::String)
                .collect(),
        ),
    );
    let mut mcp_servers = toml::map::Map::new();
    mcp_servers.insert(
        "supervisor_orchestrator".to_string(),
        toml::Value::Table(server),
    );
    let mut root = toml::map::Map::new();
    root.insert("mcp_servers".to_string(), toml::Value::Table(mcp_servers));
    toml::to_string(&toml::Value::Table(root))
        .map_err(|error| format!("序列化主管临时 MCP 配置失败：{error}"))
}

fn write_private_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("创建主管临时文件失败 {}：{error}", path.display()))?;
    restrict_private_file(path)?;
    file.write_all(contents)
        .map_err(|error| format!("写入主管临时文件失败 {}：{error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("同步主管临时文件失败 {}：{error}", path.display()))
}

#[cfg(unix)]
fn create_auth_symlink(source: &Path, destination: &Path) -> Result<(), String> {
    symlink(source, destination).map_err(|error| {
        format!(
            "创建主管 auth.json 符号链接失败 {} -> {}：{error}",
            destination.display(),
            source.display()
        )
    })
}

#[cfg(not(unix))]
fn create_auth_symlink(_source: &Path, _destination: &Path) -> Result<(), String> {
    Err("主管临时认证传送门仅支持 Unix 平台".to_string())
}

#[cfg(unix)]
fn restrict_private_dir(path: &Path) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("收紧主管临时目录权限失败 {}：{error}", path.display()))
}

#[cfg(not(unix))]
fn restrict_private_dir(_path: &Path) -> Result<(), String> {
    Err("主管临时认证目录仅支持 Unix 权限模型".to_string())
}

#[cfg(unix)]
fn restrict_private_file(path: &Path) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("收紧主管临时文件权限失败 {}：{error}", path.display()))
}

#[cfg(not(unix))]
fn restrict_private_file(_path: &Path) -> Result<(), String> {
    Err("主管临时认证文件仅支持 Unix 权限模型".to_string())
}

fn reap_supervisor_temporary_homes_once() -> Result<(), String> {
    SUPERVISOR_TEMP_HOMES_REAPED
        .get_or_init(|| reap_supervisor_temporary_homes_at(&supervisor_temporary_homes_root()))
        .clone()
}

fn reap_supervisor_temporary_homes_at(base: &Path) -> Result<(), String> {
    if !base.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(base)
        .map_err(|error| format!("枚举主管临时 home 根目录失败 {}：{error}", base.display()))?
    {
        let entry = entry.map_err(|error| format!("读取主管临时 home 条目失败：{error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("读取主管临时 home 条目类型失败：{error}"))?;
        if !file_type.is_dir() {
            continue;
        }
        let root = entry.path();
        let metadata_path = root.join(SUPERVISOR_TEMP_HOME_METADATA);
        let metadata = fs::read(&metadata_path).map_err(|error| {
            format!(
                "主管孤儿临时 home 缺审计元数据，拒绝静默清理 {}：{error}",
                root.display()
            )
        })?;
        let metadata: SupervisorTemporaryHomeMetadata =
            serde_json::from_slice(&metadata).map_err(|error| {
                format!(
                    "主管孤儿临时 home 审计元数据损坏，拒绝静默清理 {}：{error}",
                    root.display()
                )
            })?;
        SupervisorTemporaryHome::from_orphan(root, metadata).cleanup("startup_orphan_reap")?;
    }
    Ok(())
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "无".to_string()
    } else {
        values.join("；")
    }
}

fn supervisor_config(
    workflow_state_path: &Path,
    run_id: &str,
    quota_limits: SupervisorQuotaLimits,
) -> McpServerConfig {
    McpServerConfig {
        role: McpRole::SupervisorOrchestrator,
        run_id: run_id.to_string(),
        node_id: None,
        supervisor_workflow_state_path: Some(workflow_state_path.to_path_buf()),
        supervisor_quota_limits: Some(quota_limits),
    }
}

fn spawn_supervisor_session_with(
    spawner: &dyn SupervisorProcessSpawner,
    registry: &dyn SupervisorProcessRegistry,
    workflow_state_path: &Path,
    config: &McpServerConfig,
    context: &SupervisorLaunchContext,
    opening_message: String,
    command_plan: SupervisorCommandPlan,
) -> Result<
    (
        SupervisorPilotLaunchReceipt,
        Box<dyn SupervisorProcess>,
        Box<dyn SupervisorProcessRegistration>,
    ),
    String,
> {
    let (mut process, registration) = spawn_supervisor_step_with(
        spawner,
        registry,
        workflow_state_path,
        config,
        &opening_message,
        &command_plan,
    )?;
    let pid = process.pid();
    let launch = SupervisorPilotSessionLaunch {
        project_root: context.project_root.clone(),
        workflow_id: context.workflow_id.clone(),
        authorization_id: context.authorization_id.clone(),
        model_id: command_plan
            .argv
            .windows(2)
            .find(|pair| pair.first().is_some_and(|argument| argument == "--model"))
            .and_then(|pair| pair.get(1))
            .cloned()
            .unwrap_or_else(|| ACCOUNT_DEFAULT_MODEL_ID.to_string()),
        reasoning_effort: context_to_effort(&command_plan.argv).unwrap_or_default(),
        workbench_executable_path: command_plan.supervisor_mcp_command.display().to_string(),
        workbench_build_id: workbench_build_identifier(&command_plan.supervisor_mcp_command),
        supervisor_contract_version: SUPERVISOR_CONTRACT_VERSION.to_string(),
        supervisor_contract_sha256: crate::utils::hash::sha256_hex(SUPERVISOR_CONTRACT_TEMPLATE),
        worker_report_contract_sha256: crate::utils::hash::sha256_hex(
            crate::worker_report::WORKER_REPORT_CONTRACT_TEXT,
        ),
    };
    if let Err(error) = supervisor_orchestrator::record_pilot_session_started(config, &launch) {
        process.terminate();
        registration.unregister();
        return Err(error);
    }
    Ok((
        SupervisorPilotLaunchReceipt {
            run_id: config.run_id.clone(),
            pid,
            opening_message,
            argv: command_plan.argv,
            last_message_path: command_plan.last_message_path.display().to_string(),
            stderr_path: command_plan.stderr_path.display().to_string(),
            status: "running".to_string(),
        },
        process,
        registration,
    ))
}

fn workbench_build_identifier(executable: &Path) -> String {
    let metadata = fs::metadata(executable).ok();
    let byte_len = metadata.as_ref().map(|value| value.len()).unwrap_or_default();
    let modified_seconds = metadata
        .and_then(|value| value.modified().ok())
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or_default();
    let sha256 = fs::read(executable)
        .map(|bytes| crate::utils::hash::sha256_hex_bytes(&bytes))
        .unwrap_or_else(|_| "unavailable".to_string());
    format!(
        "{}@{}:bytes={byte_len}:mtime={modified_seconds}:sha256={sha256}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
}

fn spawn_supervisor_step_with(
    spawner: &dyn SupervisorProcessSpawner,
    registry: &dyn SupervisorProcessRegistry,
    workflow_state_path: &Path,
    config: &McpServerConfig,
    opening_message: &str,
    command_plan: &SupervisorCommandPlan,
) -> Result<
    (
        Box<dyn SupervisorProcess>,
        Box<dyn SupervisorProcessRegistration>,
    ),
    String,
> {
    let mut process = spawner.spawn(&command_plan, config)?;
    if let Err(error) = process.write_opening_message(opening_message) {
        process.terminate();
        return Err(error);
    }
    let pid = process.pid();
    let registration = registry.register(workflow_state_path, &config.run_id, pid);
    let registration: Box<dyn SupervisorProcessRegistration> = match process.temporary_home() {
        Some(temporary_home) => Box::new(CleanupAwareSupervisorProcessRegistration {
            registration,
            temporary_home,
        }),
        None => registration,
    };
    Ok((process, registration))
}

fn context_to_effort(argv: &[String]) -> Option<String> {
    argv.windows(2).find_map(|pair| {
        (pair.first().is_some_and(|argument| argument == "-c"))
            .then(|| pair.get(1))
            .flatten()
            .and_then(|value| value.strip_prefix("model_reasoning_effort="))
            .map(|value| value.trim_matches('"').to_string())
    })
}

fn run_supervisor_action_loop(
    spawner: &dyn SupervisorProcessSpawner,
    registry: &dyn SupervisorProcessRegistry,
    workflow_state_path: PathBuf,
    config: McpServerConfig,
    context: SupervisorLaunchContext,
    first_command_plan: SupervisorCommandPlan,
    first_process: Box<dyn SupervisorProcess>,
    first_registration: Box<dyn SupervisorProcessRegistration>,
) {
    let runtime = SupervisorActionRuntime {
        run_id: config.run_id.clone(),
        project_root: context.project_root.clone(),
        workflow_id: context.workflow_id.clone(),
        authorization_id: context.authorization_id.clone(),
        workflow_state_path: workflow_state_path.clone(),
        quota_limits: context.quota_limits,
        started_at_ms: crate::unix_timestamp_ms(),
    };
    let adapter = WorkbenchSupervisorActionAdapter;
    let action_limit = supervisor_action_limit(&context.quota_limits);
    let mut action_count = 0usize;
    let mut protocol_invalid_count = 0usize;
    let mut step = 0usize;
    let mut process = first_process;
    let mut registration = first_registration;
    let mut command_plan = first_command_plan;
    let final_exit_code = loop {
        let exit_code = process.wait().ok().flatten();
        registration.unregister();
        if exit_code != Some(0) {
            let summary = format!(
                "主管步骤 {step} 未正常结束（exit_code={:?}）；系统没有执行该步骤后的动作。",
                exit_code
            );
            let _ = record_supervisor_transport_failure(&runtime, &summary);
            break exit_code;
        }
        let last_message = match fs::read_to_string(&command_plan.last_message_path) {
            Ok(message) => message,
            Err(error) => {
                let _ = record_supervisor_transport_failure(
                    &runtime,
                    &format!(
                        "主管步骤 {step} 未写出 last_message（{}）：系统没有执行新动作。",
                        error
                    ),
                );
                break exit_code;
            }
        };
        let parsed_proposal = parse_supervisor_action_proposal(&last_message);
        let format_error = parsed_proposal.is_err();
        let terminal_action = parsed_proposal
            .as_ref()
            .map(|proposal| {
                matches!(
                    &proposal.action,
                    SupervisorActionKind::ReportUser { .. }
                        | SupervisorActionKind::RequestUserDecision { .. }
                )
            })
            .unwrap_or(false);
        let result = match execute_supervisor_last_message(&runtime, &last_message, &adapter) {
            Ok(result) => result,
            Err(error) => {
                let _ = record_supervisor_transport_failure(
                    &runtime,
                    &format!("控制核心处理主管动作失败：{error}"),
                );
                break exit_code;
            }
        };
        let may_correct_format_once = if format_error {
            protocol_invalid_count += 1;
            if let Err(error) = supervisor_orchestrator::record_pilot_protocol_invalid(
                &config,
                protocol_invalid_count,
                &result.summary,
            ) {
                let _ = record_supervisor_transport_failure(
                    &runtime,
                    &format!("主管格式错误诊断落账失败：{error}"),
                );
                break exit_code;
            }
            if protocol_invalid_count >= 2 {
                break exit_code;
            }
            true
        } else {
            false
        };
        action_count += 1;
        if matches!(result.status.as_str(), "waiting_user" | "report_invalid") && !may_correct_format_once {
            let waiting_reason = if result.status == "report_invalid" {
                format!("worker 回程格式无效，等待用户决定：{}", result.summary)
            } else {
                result.summary.clone()
            };
            let _ = supervisor_orchestrator::record_pilot_waiting_user(&config, &waiting_reason);
        }
        if terminal_action || (!result.should_continue() && !may_correct_format_once) {
            break exit_code;
        }
        if action_count >= action_limit {
            let _ = record_supervisor_action_quota_exceeded(
                &runtime,
                &format!("主管动作数已达上限 {action_limit}，等待用户决定。"),
            );
            break exit_code;
        }
        step += 1;
        let next_plan = match command_plan_for_next_step(&command_plan, &config.run_id, step) {
            Ok(plan) => plan,
            Err(error) => {
                let _ = record_supervisor_transport_failure(&runtime, &error);
                break exit_code;
            }
        };
        let next_opening_message = if may_correct_format_once {
            assemble_protocol_correction_message(&context, &result.summary, &last_message)
        } else {
            assemble_next_step_message(&context, parsed_proposal.as_ref().ok(), &result)
        };
        match spawn_supervisor_step_with(
            spawner,
            registry,
            &workflow_state_path,
            &config,
            &next_opening_message,
            &next_plan,
        ) {
            Ok((next_process, next_registration)) => {
                process = next_process;
                registration = next_registration;
                command_plan = next_plan;
            }
            Err(error) => {
                let _ = record_supervisor_transport_failure(
                    &runtime,
                    &format!("主管下一步骤启动失败：{error}"),
                );
                break None;
            }
        }
    };
    let _ = supervisor_orchestrator::record_pilot_session_finished(&config, final_exit_code);
}

#[cfg(test)]
fn finish_supervisor_session(
    process: Box<dyn SupervisorProcess>,
    registration: Box<dyn SupervisorProcessRegistration>,
    config: McpServerConfig,
) {
    let exit_code = process.wait().ok().flatten();
    registration.unregister();
    let _ = supervisor_orchestrator::record_pilot_session_finished(&config, exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FakeSpawner {
        opening_messages: Arc<Mutex<Vec<String>>>,
    }

    struct FakeProcess {
        pid: u32,
        opening_messages: Arc<Mutex<Vec<String>>>,
        exit_code: Option<i32>,
    }

    impl SupervisorProcess for FakeProcess {
        fn pid(&self) -> u32 {
            self.pid
        }

        fn write_opening_message(&mut self, opening_message: &str) -> Result<(), String> {
            self.opening_messages
                .lock()
                .expect("fake opening messages")
                .push(opening_message.to_string());
            Ok(())
        }

        fn wait(self: Box<Self>) -> Result<Option<i32>, String> {
            Ok(self.exit_code)
        }

        fn terminate(&mut self) {}
    }

    impl SupervisorProcessSpawner for FakeSpawner {
        fn spawn(
            &self,
            _plan: &SupervisorCommandPlan,
            _config: &McpServerConfig,
        ) -> Result<Box<dyn SupervisorProcess>, String> {
            Ok(Box::new(FakeProcess {
                pid: 4242,
                opening_messages: self.opening_messages.clone(),
                exit_code: Some(0),
            }))
        }
    }

    struct ScriptedSpawner {
        opening_messages: Arc<Mutex<Vec<String>>>,
        last_messages: Mutex<VecDeque<String>>,
    }

    impl ScriptedSpawner {
        fn new(last_messages: impl IntoIterator<Item = String>) -> Self {
            Self {
                opening_messages: Arc::new(Mutex::new(Vec::new())),
                last_messages: Mutex::new(last_messages.into_iter().collect()),
            }
        }
    }

    impl SupervisorProcessSpawner for ScriptedSpawner {
        fn spawn(
            &self,
            plan: &SupervisorCommandPlan,
            _config: &McpServerConfig,
        ) -> Result<Box<dyn SupervisorProcess>, String> {
            let last_message = self
                .last_messages
                .lock()
                .expect("scripted last messages")
                .pop_front()
                .ok_or_else(|| "scripted supervisor has no next last_message".to_string())?;
            if let Some(parent) = plan.last_message_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("create scripted output dir failed: {error}"))?;
            }
            fs::write(&plan.last_message_path, last_message)
                .map_err(|error| format!("write scripted last_message failed: {error}"))?;
            Ok(Box::new(FakeProcess {
                pid: 4242,
                opening_messages: self.opening_messages.clone(),
                exit_code: Some(0),
            }))
        }
    }

    #[derive(Default)]
    struct FakeRegistry {
        registrations: Arc<Mutex<Vec<(String, u32)>>>,
        unregistrations: Arc<Mutex<Vec<u32>>>,
    }

    struct FakeRegistration {
        pid: u32,
        unregistrations: Arc<Mutex<Vec<u32>>>,
    }

    impl SupervisorProcessRegistration for FakeRegistration {
        fn unregister(self: Box<Self>) {
            self.unregistrations
                .lock()
                .expect("fake unregistrations")
                .push(self.pid);
        }
    }

    impl SupervisorProcessRegistry for FakeRegistry {
        fn register(
            &self,
            _workflow_state_path: &Path,
            run_id: &str,
            pid: u32,
        ) -> Box<dyn SupervisorProcessRegistration> {
            self.registrations
                .lock()
                .expect("fake registrations")
                .push((run_id.to_string(), pid));
            Box::new(FakeRegistration {
                pid,
                unregistrations: self.unregistrations.clone(),
            })
        }
    }

    fn fixture_context() -> SupervisorLaunchContext {
        SupervisorLaunchContext {
            project_root: crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
            workflow_id: "workflow:station2:default".to_string(),
            authorization_id: "authorization:station2".to_string(),
            user_goal: "检查测试项目的只读状态".to_string(),
            user_requirement_snapshot: "用户原文：检查测试项目的只读状态".to_string(),
            approved_plan_summary: "读取关键文件并报告结论".to_string(),
            worker_acceptance_criteria: vec![
                "创建目标文件。".to_string(),
                "回读创建后的目标文件。".to_string(),
                "验证目标文件字节与用户原始需求快照中的文件名和精确内容一致。".to_string(),
            ],
            control_core_acceptance_criteria: vec![
                "本次运行使用新建的 active authorization。".to_string(),
                "本次运行绑定新建的 work item。".to_string(),
                "仅派发一个 worker；重放不得重复派发。".to_string(),
            ],
            supervisor_acceptance_criteria: vec![
                "检查合法 worker 回程和证据。".to_string(),
                "仅在证据充分时提出终标建议。".to_string(),
            ],
            allowed_read_roots: vec![crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string()],
            allowed_write_roots: vec![],
            allowed_tools: vec!["read_file".to_string()],
            allowed_checks: vec!["cargo test --lib".to_string()],
            stop_conditions: vec!["发现授权外写入需求时停下".to_string()],
            task_package_status: "已物化任务包 1 份；工作项状态：needs_binding=1。".to_string(),
            pilot_task: None,
            quota_limits: SupervisorQuotaLimits {
                max_active_workers: 2,
                max_follow_ups_per_worker: 2,
                max_runtime_minutes: 30,
            },
        }
    }

    fn fixture_request() -> SupervisorPilotLaunchRequest {
        SupervisorPilotLaunchRequest {
            project_root: crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
            workflow_id: "workflow:station2:default".to_string(),
            authorization_id: "authorization:station2".to_string(),
            model_id: None,
            reasoning_effort: "medium".to_string(),
        }
    }

    fn temporary_home_fixture(
        label: &str,
    ) -> (
        PathBuf,
        PathBuf,
        PathBuf,
        McpServerConfig,
        SupervisorCommandPlan,
    ) {
        let root = std::env::temp_dir().join(format!(
            "station2-supervisor-temporary-home-{label}-{}",
            crate::unix_timestamp_nanos()
        ));
        let homes_base = root.join("homes");
        let source_dir = root.join("source");
        fs::create_dir_all(&source_dir).expect("create temporary home fixture source directory");
        let auth_source = source_dir.join(SUPERVISOR_TEMP_HOME_AUTH);
        fs::write(&auth_source, "fixture-auth-token").expect("write fixture auth source");
        let workflow_state_path = root.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").expect("write fixture workflow state");
        let context = fixture_context();
        let config = supervisor_config(
            &workflow_state_path,
            &format!("supervisor:station2:{label}"),
            context.quota_limits,
        );
        let plan = build_supervisor_command_plan(
            &fixture_request(),
            &workflow_state_path,
            &config.run_id,
            &context.quota_limits,
            Path::new("/tmp/workbench"),
        )
        .expect("build temporary home fixture plan");
        (root, homes_base, auth_source, config, plan)
    }

    fn temporary_home_audit_events(workflow_state_path: &Path) -> Vec<Value> {
        let sidecar = crate::utils::store_paths::sidecar_path(
            workflow_state_path,
            "supervisor-orchestrator.v1.json",
            "主管编排",
        )
        .expect("temporary home audit sidecar path");
        let store: Value =
            serde_json::from_slice(&fs::read(sidecar).expect("read temporary home audit sidecar"))
                .expect("parse temporary home audit sidecar");
        store["audit_events"]
            .as_array()
            .expect("temporary home audit events")
            .clone()
    }

    #[test]
    fn opening_message_snapshot_contains_approved_contract_and_order_context() {
        let opening_message = assemble_opening_message(&fixture_context());
        assert!(opening_message.contains("执行权仍属于工作台 Syn 控制核心"));
        assert!(opening_message.contains("用户原文：检查测试项目的只读状态"));
        assert!(opening_message.contains("用户目标（方案提炼）：检查测试项目的只读状态"));
        assert!(opening_message.contains("Worker 验收（只由 worker 完成）"));
        assert!(opening_message.contains("控制核心验收（不得下放给 worker）"));
        assert!(opening_message.contains("主管验收（不得下放给 worker）"));
        assert!(opening_message.contains("授权段：authorization:station2"));
        assert!(opening_message.contains("可写根：无"));
        assert!(opening_message.contains("任务包现状：已物化任务包 1 份"));
        assert!(opening_message.contains("\"target\": {"));
        assert!(opening_message.contains("\"node_id\": \"<本单 node_id>\""));
        assert!(opening_message.contains("\"work_item_id\": \"<本单 work_item_id>\""));
        for kind in [
            "dispatch_worker",
            "inspect_worker",
            "follow_up_worker",
            "wait_worker",
            "finalize",
            "report_user",
            "request_user_decision",
        ] {
            assert!(
                opening_message.contains(&format!("\"kind\": \"{kind}\"")),
                "opening message must contain complete {kind} structure"
            );
        }
        println!("[SUPERVISOR_OPENING_SAMPLE]\n{opening_message}");
    }

    #[test]
    fn canonical_supervisor_contract_document_contains_runtime_template() {
        const CANONICAL_DOCUMENT: &str =
            include_str!("../../../../docs/plans/2026-07-11-supervisor-contract-v1-draft.md");
        let normalize = |value: &str| {
            value
                .lines()
                .filter(|line| !line.trim_start().starts_with("```"))
                .flat_map(str::split_whitespace)
                .collect::<Vec<_>>()
                .join(" ")
        };
        assert!(
            normalize(CANONICAL_DOCUMENT).contains(&normalize(SUPERVISOR_CONTRACT_TEMPLATE)),
            "主管契约文档与运行时模板发生漂移"
        );
    }

    #[test]
    fn station3a_completed_inspection_must_advance_instead_of_repeating() {
        let previous = parse_supervisor_action_proposal(
            r#"{"schema_version":"supervisor_action_proposal.v1","kind":"inspect_worker","worker_id":"worker-1","reason":"检查回程","expected_result":"获得证据"}"#,
        )
        .expect("parse inspect proposal");
        let result = SupervisorActionResultV1 {
            action_id: Some("action:inspect:1".to_string()),
            status: "completed".to_string(),
            summary: "合法 worker 回程已落账".to_string(),
            worker_id: Some("worker-1".to_string()),
            adapter_id: Some("codex-local-authorized-dispatch".to_string()),
            evidence_present: true,
        };

        let message = assemble_next_step_message(&fixture_context(), Some(&previous), &result);
        assert!(message.contains("同一 worker `worker-1` 的 inspect_worker 已完成"));
        assert!(message.contains("禁止再次输出同一 inspect_worker"));
        assert!(message.contains("下一步提议 finalize"));
        assert!(message.contains("提议 follow_up_worker"));
        assert!(message.contains("提议 request_user_decision"));
    }

    #[test]
    fn station3a_requires_all_three_native_acceptance_roles() {
        let worker = vec!["worker 验收".to_string()];
        let control_core = vec!["控制核心验收".to_string()];
        let supervisor = vec!["主管验收".to_string()];
        validate_supervisor_pilot_role_criteria(&worker, &control_core, &supervisor)
            .expect("三类职责非空时应通过");

        for (missing, result) in [
            (
                "worker_acceptance_criteria",
                validate_supervisor_pilot_role_criteria(&[], &control_core, &supervisor),
            ),
            (
                "control_core_acceptance_criteria",
                validate_supervisor_pilot_role_criteria(&worker, &[], &supervisor),
            ),
            (
                "supervisor_acceptance_criteria",
                validate_supervisor_pilot_role_criteria(&worker, &control_core, &[]),
            ),
        ] {
            let error = result.expect_err("任一职责为空都必须拒绝物化");
            assert!(error.contains(missing), "{error}");
        }
    }

    #[test]
    fn station3a_v5_worker_objective_uses_only_worker_acceptance() {
        let criteria = vec![
            "创建 station3a-control-core-proof-v5.txt。".to_string(),
            "精确写入 UTF-8 内容 station3a control core proof v5 passed!，不得添加末尾换行。".to_string(),
            "回读目标文件。".to_string(),
            "验证内容、39 bytes 与无末尾换行。".to_string(),
            "返回执行证据。".to_string(),
        ];
        let first = supervisor_pilot_worker_objective(&criteria);
        let second = supervisor_pilot_worker_objective(&criteria);
        assert_eq!(first, second);
        assert!(first.contains("station3a-control-core-proof-v5.txt"));
        assert!(first.contains("station3a control core proof v5 passed!"));
        assert!(first.contains("39 bytes"));
        assert!(first.contains("执行下列 worker 验收"));
        for forbidden in [
            "新建 authorization",
            "新建 work item",
            "新建 supervisor run",
            "唯一 worker",
            "主管终标",
            "检查 UI 入口",
        ] {
            assert!(!first.contains(forbidden), "worker objective leaked: {forbidden}");
        }
        for criterion in criteria {
            assert!(first.contains(&criterion));
        }
        assert!(
            supervisor_pilot_worker_task_title(&[
                "创建 station3a-control-core-proof-v5.txt。".to_string()
            ])
            .contains("station3a-control-core-proof-v5.txt"),
            "worker task title may only derive from worker criteria"
        );
        let task_package_goals = crate::worker_report::build_goals_with_contract(&first, &[]);
        assert_eq!(
            task_package_goals.first(),
            Some(&first),
            "物化任务包必须把 worker objective 作为首个 goal"
        );
        assert!(
            task_package_goals[0].contains("station3a-control-core-proof-v5.txt"),
            "worker 的文件名和精确内容不得在任务包中丢失"
        );
        assert_eq!(
            task_package_goals
                .iter()
                .filter(|goal| goal.contains("受阻 blocked 的完整示例"))
                .count(),
            1,
            "worker prompt 只能包含一份当前 worker 回程契约"
        );
    }

    #[test]
    fn station3a_v3_protocol_correction_uses_the_original_action_shape() {
        let malformed_inspect = r#"{"schema_version":"supervisor_action_proposal.v1","kind":"inspect_worker","target":{"worker_id":"worker-1"},"reason":"read","expected_result":"evidence"}"#;
        let correction = assemble_protocol_correction_message(
            &fixture_context(),
            "protocol_invalid: target 不允许",
            malformed_inspect,
        );
        let selected_example = correction
            .split("===== 正确 JSON 示例 =====")
            .nth(1)
            .expect("selected example section");
        assert!(selected_example.contains("\"kind\": \"inspect_worker\""));
        assert!(selected_example.contains("\"worker_id\": \"<已登记 worker_id>\""));
        assert!(!selected_example.contains("\"kind\": \"dispatch_worker\""));
    }

    #[test]
    fn station3a_two_format_errors_are_diagnosed_then_stop_waiting_user() {
        let temp = std::env::temp_dir().join(format!(
            "station3a-protocol-retry-{}",
            crate::unix_timestamp_nanos()
        ));
        fs::create_dir_all(&temp).expect("create protocol retry fixture");
        let workflow_state_path = temp.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, r#"{"revision":1}"#).expect("write workflow state");
        let context = fixture_context();
        let config = supervisor_config(
            &workflow_state_path,
            "supervisor:station3a:protocol-retry",
            context.quota_limits,
        );
        let command_plan = build_supervisor_command_plan(
            &fixture_request(),
            &workflow_state_path,
            &config.run_id,
            &context.quota_limits,
            Path::new("/tmp/workbench"),
        )
        .expect("build protocol retry plan");
        let wrong_top_level = r#"{"schema_version":"supervisor_action_proposal.v1","kind":"dispatch_worker","node_id":"wrong","work_item_id":"wrong","reason":"x","expected_result":"y"}"#.to_string();
        let spawner = ScriptedSpawner::new([wrong_top_level, "not json".to_string()]);
        let registry = FakeRegistry::default();
        let (_, first_process, first_registration) = spawn_supervisor_session_with(
            &spawner,
            &registry,
            &workflow_state_path,
            &config,
            &context,
            assemble_opening_message(&context),
            command_plan.clone(),
        )
        .expect("spawn first scripted supervisor step");

        run_supervisor_action_loop(
            &spawner,
            &registry,
            workflow_state_path.clone(),
            config.clone(),
            context,
            command_plan,
            first_process,
            first_registration,
        );

        let read_model =
            supervisor_orchestrator::load_pilot_read_model(&config).expect("protocol retry read model");
        assert_eq!(read_model.launch_status, "waiting_user");
        assert_eq!(
            read_model.termination_reason,
            "主管连续两次输出格式错误，当前无效动作未执行。本单未执行。"
        );
        let diagnostics = read_model
            .audit_events
            .iter()
            .filter(|event| event.tool == "supervisor_action_protocol")
            .collect::<Vec<_>>();
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].result_status, "protocol_invalid");
        assert_eq!(diagnostics[1].result_status, "waiting_user");
        assert!(diagnostics[1].result_summary.contains("第二次错误："));
        assert!(!diagnostics
            .iter()
            .any(|event| event.result_summary.contains("user_cancelled")));

        let opening_messages = spawner
            .opening_messages
            .lock()
            .expect("scripted opening messages");
        assert_eq!(opening_messages.len(), 2);
        assert!(opening_messages[1].contains("具体错误："));
        assert!(opening_messages[1].contains("这是唯一一次格式纠正机会"));
        assert!(opening_messages[1].contains("\"target\": {"));

        let orchestrator_sidecar = crate::utils::store_paths::sidecar_path(
            &workflow_state_path,
            "supervisor-orchestrator.v1.json",
            "主管编排",
        )
        .expect("orchestrator sidecar");
        let orchestrator_store: Value = serde_json::from_slice(
            &fs::read(orchestrator_sidecar).expect("read orchestrator sidecar"),
        )
        .expect("parse orchestrator sidecar");
        assert!(orchestrator_store["sessions"][0]["workers"]
            .as_array()
            .is_some_and(Vec::is_empty));

        let action_sidecar = crate::utils::store_paths::sidecar_path(
            &workflow_state_path,
            "supervisor-action-control.v1.json",
            "supervisor action control",
        )
        .expect("action controller sidecar");
        let action_store: Value =
            serde_json::from_slice(&fs::read(action_sidecar).expect("read action sidecar"))
                .expect("parse action sidecar");
        let actions = action_store["actions"].as_array().expect("action records");
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().all(|action| {
            action["kind"] == "system_protocol_invalid"
                && action["execution_status"] == "protocol_invalid"
        }));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn station3a_write_scope_accepts_only_exact_fixed_test_project_root() {
        let test_root = crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        assert!(ensure_supervisor_pilot_write_scope(test_root, &[]).is_ok());
        assert!(ensure_supervisor_pilot_write_scope(test_root, &[test_root.to_string()]).is_ok());
        for rejected in [
            "/tmp/not-the-test-project",
            "/Users/yoyi/codex-workflow-mario-test/subdir",
            "/Users/yoyi/codex-workflow-mario-test/../real-project",
        ] {
            assert!(
                ensure_supervisor_pilot_write_scope(test_root, &[rejected.to_string()]).is_err()
            );
        }
    }

    // 站 3b/4 案发测试：同一 mario 根只认 3b 零写根或 4 唯一同根写根；其它形状一律拒。
    // 非测试非 mario 项目维持 blocked；主管 argv 在两站下都仍固定测试根 + read-only + 零 --add-dir。
    #[test]
    fn station3b_and_station4_write_scope_are_exact_and_supervisor_stays_readonly() {
        let station3b_root = crate::STATION_3B_READONLY_PROJECT_ROOT;
        let station4_root = crate::STATION_4_WRITE_PROJECT_ROOT;
        assert_eq!(station3b_root, station4_root, "同根但不是同一授权语义");
        assert!(ensure_supervisor_pilot_write_scope(station3b_root, &[]).is_ok());
        assert!(ensure_supervisor_pilot_write_scope(station4_root, &[station4_root.to_string()]).is_ok());
        for write_root in [
            crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            "/tmp/anything",
        ] {
            assert!(
                ensure_supervisor_pilot_write_scope(station3b_root, &[write_root.to_string()])
                    .is_err(),
                "mario 项目异形写根 {write_root} 必须拒绝"
            );
        }
        assert!(ensure_supervisor_pilot_write_scope(
            station4_root,
            &[station4_root.to_string(), station4_root.to_string()]
        )
        .is_err());
        // 其它真实项目根：两闸都不认。
        assert!(ensure_supervisor_pilot_write_scope("/Users/yoyi/gameai/crazytown", &[]).is_err());

        // argv 终验：mario project_root 放行，但主管 -C 仍必须是固定测试项目根。
        let argv_with = |cwd: &str, sandbox: &str| -> Vec<String> {
            [
                "exec",
                "--ephemeral",
                "-C",
                cwd,
                "--sandbox",
                sandbox,
                "--json",
                "-c",
                "features.multi_agent=false",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect()
        };
        let good_argv = argv_with(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "read-only");
        assert!(validate_supervisor_argv(&good_argv, station3b_root).is_ok());
        assert!(
            !good_argv.iter().any(|argument| argument == "--add-dir"),
            "主管只读 argv 不应有写目录：{good_argv:?}"
        );
        // -C 指向 mario 项目 → 拒（主管不进真实项目目录）。
        assert!(
            validate_supervisor_argv(&argv_with(station3b_root, "read-only"), station3b_root)
                .is_err()
        );
        // workspace-write → 拒。
        assert!(validate_supervisor_argv(
            &argv_with(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "workspace-write"),
            station3b_root
        )
        .is_err());
        // 其它真实项目 project_root → 拒。
        assert!(validate_supervisor_argv(&good_argv, "/Users/yoyi/gameai/crazytown").is_err());

        let station4_plan = build_supervisor_command_plan(
            &SupervisorPilotLaunchRequest {
                project_root: station4_root.to_string(),
                ..fixture_request()
            },
            Path::new("/tmp/station4-supervisor-workflow-state.json"),
            "supervisor:station4:argv",
            &fixture_context().quota_limits,
            Path::new("/tmp/workbench"),
        )
        .expect("站 4 主管计划仍应可组装为只读计划");
        assert!(argv_contains_pair(&station4_plan.argv, "--sandbox", "read-only"));
        assert!(
            !station4_plan
                .argv
                .iter()
                .any(|argument| argument == "--add-dir"),
            "站 4 只允许 worker 获写根，主管 argv 不得有 --add-dir：{:?}",
            station4_plan.argv
        );
    }

    #[test]
    fn station3a_planned_task_id_is_unique_per_authorization_before_stable_id_truncation() {
        let shared_prefix = "plan-auth:project-users-yoyi-codex-workflow-mario-test-workflow-users-yoyi-codex-workflow-mario-test-default";
        let first = supervisor_pilot_planned_task_id(&format!("{shared_prefix}:1001"));
        let second = supervisor_pilot_planned_task_id(&format!("{shared_prefix}:1002"));
        assert_ne!(first, second);
        assert_ne!(crate::stable_id(&first), crate::stable_id(&second));
    }

    #[test]
    fn station3a_opening_message_discloses_authorized_write_root() {
        let mut context = fixture_context();
        context.allowed_write_roots = vec![crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string()];
        context.pilot_task = Some(SupervisorPilotTaskReference {
            node_id: "workflow:fixture:node:codex-dev".to_string(),
            work_item_id: "work-item:fixture".to_string(),
            allowed_write: vec![crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string()],
        });
        let opening_message = assemble_opening_message(&context);
        assert!(opening_message.contains(&format!(
            "可写根：{}",
            crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT
        )));
        assert!(opening_message.contains("node_id=workflow:fixture:node:codex-dev"));
        assert!(opening_message.contains("work_item_id=work-item:fixture"));
        assert!(opening_message.contains("只输出第一个 `SupervisorActionProposalV1` JSON"));
        assert!(!opening_message.contains("mcp__supervisor_orchestrator__dispatch_worker"));
    }

    #[test]
    fn argv_requires_readonly_isolated_codex_home_no_bypass_and_test_project_lock() {
        let request = fixture_request();
        let limits = fixture_context().quota_limits;
        let plan = build_supervisor_command_plan(
            &request,
            Path::new("/tmp/station2-workflow-state.json"),
            "supervisor:station2:1",
            &limits,
            Path::new("/tmp/workbench"),
        )
        .expect("build plan");
        assert!(argv_contains_pair(&plan.argv, "--sandbox", "read-only"));
        assert!(!plan
            .argv
            .iter()
            .any(|argument| argument == "--ignore-user-config"));
        assert!(argv_contains_pair(
            &plan.argv,
            "-C",
            crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT
        ));
        assert!(!plan.argv.iter().any(|argument| argument == "--model"));
        let expected_output_dir = Path::new("/tmp")
            .join("runtime-artifacts")
            .join("supervisor")
            .join(crate::utils::hash::short_hash("supervisor:station2:1"));
        assert!(argv_contains_pair(
            &plan.argv,
            "--output-last-message",
            expected_output_dir
                .join("step-0.last-message.txt")
                .to_string_lossy()
                .as_ref()
        ));
        assert_eq!(
            plan.stderr_path,
            expected_output_dir.join("step-0.stderr.txt")
        );
        assert!(!plan.last_message_path.to_string_lossy().contains(':'));
        assert!(plan
            .argv
            .iter()
            .any(|argument| argument == "model_reasoning_effort=\"medium\""));
        assert!(plan
            .argv
            .iter()
            .any(|argument| argument == "features.multi_agent=false"));
        for forbidden in [
            "--full-auto",
            "--dangerously-bypass-approvals-and-sandbox",
            "--approval-policy=never",
            "full-auto",
        ] {
            let mut rejected = plan.argv.clone();
            rejected.push(forbidden.to_string());
            assert!(validate_supervisor_argv(&rejected, &request.project_root).is_err());
        }
        assert!(build_supervisor_command_plan(
            &SupervisorPilotLaunchRequest {
                project_root: "/tmp/not-the-test-project".to_string(),
                ..fixture_request()
            },
            Path::new("/tmp/station2-workflow-state.json"),
            "supervisor:station2:1",
            &limits,
            Path::new("/tmp/workbench"),
        )
        .is_err());
        let custom_model_plan = build_supervisor_command_plan(
            &SupervisorPilotLaunchRequest {
                model_id: Some("terra".to_string()),
                ..fixture_request()
            },
            Path::new("/tmp/station2-workflow-state.json"),
            "supervisor:station2:custom-model",
            &limits,
            Path::new("/tmp/workbench"),
        )
        .expect("build custom model plan");
        assert!(argv_contains_pair(
            &custom_model_plan.argv,
            "--model",
            "terra"
        ));
        println!("[SUPERVISOR_ARGV] {:?}", plan.argv);
    }

    #[test]
    fn temporary_home_is_private_mcp_only_and_unregister_cleans() {
        let (fixture_root, homes_base, auth_source, config, plan) =
            temporary_home_fixture("private-mcp-only");
        let workflow_state_path = config
            .supervisor_workflow_state_path
            .clone()
            .expect("fixture workflow state path");
        let temporary_home =
            SupervisorTemporaryHome::create_at(&homes_base, &auth_source, &plan, &config)
                .expect("create temporary home");
        let temporary_home_path = temporary_home.root().to_path_buf();
        assert!(fs::symlink_metadata(&temporary_home.auth_path)
            .expect("inspect temporary auth link")
            .file_type()
            .is_symlink());
        assert_eq!(
            fs::read_link(&temporary_home.auth_path).expect("read temporary auth link"),
            auth_source
        );
        let config_text = fs::read_to_string(temporary_home_path.join(SUPERVISOR_TEMP_HOME_CONFIG))
            .expect("read temporary MCP config");
        let config_toml: toml::Value =
            toml::from_str(&config_text).expect("parse temporary MCP config");
        let config_table = config_toml.as_table().expect("temporary MCP config table");
        assert_eq!(config_table.len(), 1);
        assert!(!config_table.contains_key("approval_policy"));
        assert!(!config_table.contains_key("approvals_reviewer"));
        let servers = config_table["mcp_servers"]
            .as_table()
            .expect("temporary MCP servers table");
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("supervisor_orchestrator"));
        assert!(!config_text.contains("fixture-auth-token"));
        #[cfg(unix)]
        {
            assert_eq!(
                fs::metadata(&temporary_home_path)
                    .expect("temporary home permissions")
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
            assert_eq!(
                fs::metadata(temporary_home_path.join(SUPERVISOR_TEMP_HOME_CONFIG))
                    .expect("temporary config permissions")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        let unregistrations = Arc::new(Mutex::new(Vec::new()));
        Box::new(CleanupAwareSupervisorProcessRegistration {
            registration: Box::new(FakeRegistration {
                pid: 7331,
                unregistrations: unregistrations.clone(),
            }),
            temporary_home,
        })
        .unregister();
        assert!(!temporary_home_path.exists());
        assert_eq!(
            unregistrations.lock().expect("unregistrations").as_slice(),
            [7331]
        );
        let audit_events = temporary_home_audit_events(&workflow_state_path);
        assert!(audit_events.iter().any(|event| {
            event["tool"] == "supervisor_temporary_codex_home"
                && event["parameter_summary"]
                    .as_str()
                    .is_some_and(|summary| summary.contains("action=created"))
        }));
        assert!(audit_events.iter().any(|event| {
            event["tool"] == "supervisor_temporary_codex_home"
                && event["parameter_summary"]
                    .as_str()
                    .is_some_and(|summary| summary.contains("action=cleaned"))
                && event["result_status"] == "accepted"
        }));
        let _ = fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn temporary_home_replaced_auth_emits_warning_and_removes_refresh() {
        let (fixture_root, homes_base, auth_source, config, plan) =
            temporary_home_fixture("token-refresh");
        let workflow_state_path = config
            .supervisor_workflow_state_path
            .clone()
            .expect("fixture workflow state path");
        let temporary_home =
            SupervisorTemporaryHome::create_at(&homes_base, &auth_source, &plan, &config)
                .expect("create temporary home");
        let temporary_home_path = temporary_home.root().to_path_buf();
        fs::remove_file(&temporary_home.auth_path).expect("replace temporary auth symlink");
        fs::write(&temporary_home.auth_path, "refreshed-token")
            .expect("write replaced temporary auth");
        temporary_home
            .cleanup("test_token_refresh")
            .expect("clean replaced temporary auth");
        assert!(!temporary_home_path.exists());
        let audit_events = temporary_home_audit_events(&workflow_state_path);
        assert!(audit_events.iter().any(|event| {
            event["tool"] == "supervisor_temporary_codex_home"
                && event["parameter_summary"]
                    .as_str()
                    .is_some_and(|summary| summary.contains("trigger=test_token_refresh"))
                && event["result_summary"] == "主管会话期间 token 被刷新,如遇登录失效请重登 codex"
                && event["result_status"] == "warning"
        }));
        let _ = fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn startup_orphan_sweep_cleans_registered_temporary_home() {
        let (fixture_root, homes_base, auth_source, config, plan) =
            temporary_home_fixture("orphan-reap");
        let workflow_state_path = config
            .supervisor_workflow_state_path
            .clone()
            .expect("fixture workflow state path");
        let orphan = create_private_supervisor_home_dir(&homes_base, &config.run_id)
            .expect("create orphan temporary home");
        write_private_file(
            &orphan.join(SUPERVISOR_TEMP_HOME_CONFIG),
            supervisor_mcp_config_toml(&plan)
                .expect("serialize orphan temporary MCP config")
                .as_bytes(),
        )
        .expect("write orphan temporary MCP config");
        let metadata = serde_json::to_vec(&SupervisorTemporaryHomeMetadata {
            run_id: config.run_id.clone(),
            workflow_state_path: workflow_state_path.clone(),
        })
        .expect("serialize orphan temporary metadata");
        write_private_file(&orphan.join(SUPERVISOR_TEMP_HOME_METADATA), &metadata)
            .expect("write orphan temporary metadata");
        create_auth_symlink(&auth_source, &orphan.join(SUPERVISOR_TEMP_HOME_AUTH))
            .expect("create orphan temporary auth link");
        reap_supervisor_temporary_homes_at(&homes_base).expect("reap orphan temporary home");
        assert!(!orphan.exists());
        let audit_events = temporary_home_audit_events(&workflow_state_path);
        assert!(audit_events.iter().any(|event| {
            event["tool"] == "supervisor_temporary_codex_home"
                && event["parameter_summary"]
                    .as_str()
                    .is_some_and(|summary| summary.contains("trigger=startup_orphan_reap"))
                && event["result_status"] == "accepted"
        }));
        let _ = fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn temporary_home_drop_cleans_after_unexpected_owner_release() {
        let (fixture_root, homes_base, auth_source, config, plan) =
            temporary_home_fixture("drop-cleanup");
        let temporary_home =
            SupervisorTemporaryHome::create_at(&homes_base, &auth_source, &plan, &config)
                .expect("create temporary home");
        let temporary_home_path = temporary_home.root().to_path_buf();
        drop(temporary_home);
        assert!(!temporary_home_path.exists());
        let _ = fs::remove_dir_all(fixture_root);
    }

    #[test]
    fn mock_spawn_registers_then_unregisters_with_existing_process_registry_lifecycle() {
        let temp = std::env::temp_dir().join(format!(
            "station2-supervisor-launcher-{}",
            crate::unix_timestamp_nanos()
        ));
        std::fs::create_dir_all(&temp).expect("create temp state directory");
        let workflow_state_path = temp.join("workflow-state.v0.json");
        let context = fixture_context();
        let config = supervisor_config(
            &workflow_state_path,
            "supervisor:station2:mock",
            context.quota_limits,
        );
        let command_plan = build_supervisor_command_plan(
            &fixture_request(),
            &workflow_state_path,
            &config.run_id,
            &context.quota_limits,
            Path::new("/tmp/workbench"),
        )
        .expect("build plan");
        let spawner = FakeSpawner::default();
        let registry = FakeRegistry::default();
        let (receipt, process, registration) = spawn_supervisor_session_with(
            &spawner,
            &registry,
            &workflow_state_path,
            &config,
            &context,
            assemble_opening_message(&context),
            command_plan,
        )
        .expect("mock launch");
        assert_eq!(receipt.pid, 4242);
        assert_eq!(
            registry
                .registrations
                .lock()
                .expect("registrations")
                .as_slice(),
            [("supervisor:station2:mock".to_string(), 4242)]
        );
        finish_supervisor_session(process, registration, config.clone());
        assert_eq!(
            registry
                .unregistrations
                .lock()
                .expect("unregistrations")
                .as_slice(),
            [4242]
        );
        let read_model =
            supervisor_orchestrator::load_pilot_read_model(&config).expect("pilot read model");
        assert_eq!(read_model.launch_status, "exited");
        assert!(read_model.metrics.ledger_replay_ready);
        let _ = std::fs::remove_dir_all(temp);
    }
}
