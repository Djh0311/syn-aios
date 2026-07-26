//! Host-owned, short-lived relay for `knowledge_open`.
//!
//! The supervisor MCP server is a separate stdio child process, so a successful
//! `knowledge_open` needs one bounded return path through the desktop host.
//! This module deliberately carries only one validated Markdown relative path
//! and an acknowledgement.  It does not persist a vault path, body, binding,
//! command, URL, or any other second source of truth.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

const RELAY_SCHEMA_VERSION: u8 = 1;
const MAX_RELAY_FRAME_BYTES: usize = 4 * 1024;
const RELAY_GRANT_LEASE: Duration = Duration::from_secs(60);
const UI_ACK_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_ACTIVE_CONNECTIONS: usize = 8;
const RELAY_EVENT_NAME: &str = "knowledge-open-intent";

/// Identity that a host-created grant is tied to.  All fields are host facts;
/// no caller can construct a production grant from a Tauri/MCP argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RelayBindingIdentity {
    run_id: String,
    turn_id: String,
    project_id: String,
}

impl RelayBindingIdentity {
    pub(crate) fn new(run_id: &str, turn_id: &str, project_id: &str) -> Self {
        Self {
            run_id: run_id.to_string(),
            turn_id: turn_id.to_string(),
            project_id: project_id.to_string(),
        }
    }
}

/// A redacted child-only configuration.  It is passed by the host through the
/// fixed supervisor MCP argv; it is never serializable as a UI/tool payload.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct KnowledgeOpenRelayMcpConfig {
    endpoint: String,
    grant: String,
    turn_id: String,
    project_id: String,
}

impl fmt::Debug for KnowledgeOpenRelayMcpConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KnowledgeOpenRelayMcpConfig { redacted: true }")
    }
}

impl KnowledgeOpenRelayMcpConfig {
    pub(crate) fn from_mcp_arguments(
        endpoint: String,
        grant: String,
        turn_id: String,
        project_id: String,
    ) -> Result<Self, String> {
        if !valid_absolute_socket_path(&endpoint)
            || !valid_secret_token(&grant)
            || !valid_internal_identifier(&turn_id)
            || !valid_internal_identifier(&project_id)
        {
            return Err("knowledge_open_relay_config_invalid".to_string());
        }
        Ok(Self {
            endpoint,
            grant,
            turn_id,
            project_id,
        })
    }

    pub(crate) fn append_mcp_args(&self, args: &mut Vec<String>) {
        args.extend([
            "--knowledge-open-relay-endpoint".to_string(),
            self.endpoint.clone(),
            "--knowledge-open-relay-grant".to_string(),
            self.grant.clone(),
            "--knowledge-open-relay-turn-id".to_string(),
            self.turn_id.clone(),
            "--knowledge-open-relay-project-id".to_string(),
            self.project_id.clone(),
        ]);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RelayDispatchStatus {
    AwaitingUiAck,
    Opened,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RelayDispatchSnapshot {
    intent_id: String,
    status: RelayDispatchStatus,
}

impl RelayDispatchSnapshot {
    pub(crate) fn intent_id(&self) -> &str {
        &self.intent_id
    }

    #[cfg(test)]
    pub(crate) fn status(&self) -> RelayDispatchStatus {
        self.status
    }

    #[cfg(test)]
    pub(crate) fn opened(&self) -> bool {
        self.status == RelayDispatchStatus::Opened
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RelayAckOutcome {
    Opened,
    Rejected,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RelayBindingStateForTest {
    Active,
    Starting,
    Failed,
    Terminated,
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct RelayIntentRequest {
    grant: String,
    identity: RelayBindingIdentity,
    relative_path: String,
    frame_bytes: usize,
}

#[cfg(test)]
impl RelayIntentRequest {
    pub(crate) fn new(
        grant: String,
        run_id: &str,
        turn_id: &str,
        project_id: &str,
        relative_path: &str,
    ) -> Self {
        Self {
            grant,
            identity: RelayBindingIdentity::new(run_id, turn_id, project_id),
            relative_path: relative_path.to_string(),
            frame_bytes: 1,
        }
    }

    pub(crate) fn oversize_for_test(grant: String, identity: RelayBindingIdentity) -> Self {
        Self {
            grant,
            identity,
            relative_path: "research/OpenMe.md".to_string(),
            frame_bytes: MAX_RELAY_FRAME_BYTES + 1,
        }
    }
}

#[derive(Clone)]
struct RelayGrant {
    identity: RelayBindingIdentity,
    expires_at: Instant,
}

#[derive(Clone)]
struct PendingRelayIntent {
    grant: String,
    identity: RelayBindingIdentity,
    relative_path: String,
    expires_at: Instant,
}

#[derive(Default)]
struct RelayCore {
    grants: BTreeMap<String, RelayGrant>,
    pending: BTreeMap<String, PendingRelayIntent>,
}

#[derive(Default)]
struct RelayCleanup {
    grants: Vec<String>,
    intents: Vec<String>,
}

impl RelayCore {
    fn issue(
        &mut self,
        grant: String,
        identity: RelayBindingIdentity,
        now: Instant,
    ) -> Result<(), String> {
        self.expire(now);
        if self.grants.contains_key(&grant)
            || self.pending.values().any(|pending| pending.grant == grant)
        {
            return Err("knowledge_open_relay_grant_collision".to_string());
        }
        self.grants.insert(
            grant,
            RelayGrant {
                identity,
                expires_at: now + RELAY_GRANT_LEASE,
            },
        );
        Ok(())
    }

    fn inspect(
        &mut self,
        grant: &str,
        identity: &RelayBindingIdentity,
        now: Instant,
    ) -> Result<(), String> {
        self.expire(now);
        let Some(existing) = self.grants.get(grant) else {
            return Err("knowledge_open_relay_grant_unavailable".to_string());
        };
        if existing.identity != *identity || existing.expires_at <= now {
            return Err("knowledge_open_relay_grant_rejected".to_string());
        }
        if self.pending.values().any(|pending| pending.grant == grant) {
            return Err("knowledge_open_relay_intent_already_pending".to_string());
        }
        Ok(())
    }

    fn accept(
        &mut self,
        grant: &str,
        identity: &RelayBindingIdentity,
        relative_path: String,
        intent_id: String,
        now: Instant,
    ) -> Result<RelayDispatchSnapshot, String> {
        self.inspect(grant, identity, now)?;
        if !valid_intent_id(&intent_id) || self.pending.contains_key(&intent_id) {
            return Err("knowledge_open_relay_intent_invalid".to_string());
        }
        // A grant is single-use.  Moving it to pending makes replay impossible
        // before, during, and after the UI acknowledgement.
        self.grants.remove(grant);
        self.pending.insert(
            intent_id.clone(),
            PendingRelayIntent {
                grant: grant.to_string(),
                identity: identity.clone(),
                relative_path,
                expires_at: now + UI_ACK_TIMEOUT,
            },
        );
        Ok(RelayDispatchSnapshot {
            intent_id,
            status: RelayDispatchStatus::AwaitingUiAck,
        })
    }

    fn acknowledge(
        &mut self,
        intent_id: &str,
        relative_path: &str,
        outcome: RelayAckOutcome,
        now: Instant,
    ) -> Result<RelayDispatchSnapshot, String> {
        self.expire(now);
        let Some(pending) = self.pending.get(intent_id) else {
            return Err("knowledge_open_relay_intent_unavailable".to_string());
        };
        if pending.relative_path != relative_path || pending.expires_at <= now {
            return Err("knowledge_open_relay_ack_rejected".to_string());
        }
        self.pending.remove(intent_id);
        Ok(RelayDispatchSnapshot {
            intent_id: intent_id.to_string(),
            status: match outcome {
                RelayAckOutcome::Opened => RelayDispatchStatus::Opened,
                RelayAckOutcome::Rejected => RelayDispatchStatus::Rejected,
            },
        })
    }

    fn cancel(&mut self, intent_id: &str) {
        self.pending.remove(intent_id);
    }

    fn revoke_run(&mut self, run_id: &str) -> RelayCleanup {
        let grants = self
            .grants
            .iter()
            .filter_map(|(grant, item)| (item.identity.run_id == run_id).then(|| grant.clone()))
            .collect::<Vec<_>>();
        for grant in &grants {
            self.grants.remove(grant);
        }
        let intents = self
            .pending
            .iter()
            .filter_map(|(intent_id, item)| {
                (item.identity.run_id == run_id).then(|| intent_id.clone())
            })
            .collect::<Vec<_>>();
        for intent_id in &intents {
            self.pending.remove(intent_id);
        }
        RelayCleanup { grants, intents }
    }

    fn expire(&mut self, now: Instant) -> RelayCleanup {
        let grants = self
            .grants
            .iter()
            .filter_map(|(grant, item)| (item.expires_at <= now).then(|| grant.clone()))
            .collect::<Vec<_>>();
        for grant in &grants {
            self.grants.remove(grant);
        }
        let intents = self
            .pending
            .iter()
            .filter_map(|(intent_id, item)| (item.expires_at <= now).then(|| intent_id.clone()))
            .collect::<Vec<_>>();
        for intent_id in &intents {
            self.pending.remove(intent_id);
        }
        RelayCleanup { grants, intents }
    }
}

#[cfg(test)]
/// Pure contract harness used by the R0 tests.  Production dispatch uses the
/// same `RelayCore`; the harness has no socket, Tauri state, or filesystem.
pub(crate) struct RelayTestHarness {
    core: RelayCore,
    identity: RelayBindingIdentity,
    binding_state: RelayBindingStateForTest,
    now: Instant,
    sequence: u64,
}

#[cfg(test)]
impl RelayTestHarness {
    pub(crate) fn active(identity: RelayBindingIdentity) -> Self {
        Self {
            core: RelayCore::default(),
            identity,
            binding_state: RelayBindingStateForTest::Active,
            now: Instant::now(),
            sequence: 0,
        }
    }

    pub(crate) fn with_binding_state_for_test(
        identity: RelayBindingIdentity,
        binding_state: RelayBindingStateForTest,
    ) -> Self {
        Self {
            core: RelayCore::default(),
            identity,
            binding_state,
            now: Instant::now(),
            sequence: 0,
        }
    }

    pub(crate) fn issue_grant(&mut self) -> String {
        self.sequence += 1;
        let grant = format!("grant:test:{:016x}", self.sequence);
        self.core
            .issue(grant.clone(), self.identity.clone(), self.now)
            .expect("test grants are unique");
        grant
    }

    pub(crate) fn accept_intent(
        &mut self,
        request: RelayIntentRequest,
    ) -> Result<RelayDispatchSnapshot, String> {
        if self.binding_state != RelayBindingStateForTest::Active {
            return Err("knowledge_open_relay_binding_not_active".to_string());
        }
        if request.frame_bytes > MAX_RELAY_FRAME_BYTES {
            return Err("knowledge_open_relay_frame_too_large".to_string());
        }
        self.sequence += 1;
        self.core.accept(
            &request.grant,
            &request.identity,
            request.relative_path,
            format!("intent:test:{:016x}", self.sequence),
            self.now,
        )
    }

    pub(crate) fn acknowledge(
        &mut self,
        intent_id: &str,
        relative_path: &str,
        outcome: RelayAckOutcome,
    ) -> Result<RelayDispatchSnapshot, String> {
        self.core
            .acknowledge(intent_id, relative_path, outcome, self.now)
    }

    pub(crate) fn replay_intent_for_test(&mut self, intent_id: &str) -> Result<(), String> {
        if self.core.pending.contains_key(intent_id) {
            Err("knowledge_open_relay_replay_rejected".to_string())
        } else {
            Err("knowledge_open_relay_intent_unavailable".to_string())
        }
    }

    pub(crate) fn expire_for_test(&mut self) {
        self.now += RELAY_GRANT_LEASE + UI_ACK_TIMEOUT;
        self.core.expire(self.now);
    }

    pub(crate) fn revoke_run_for_test(&mut self) {
        let run_id = self.identity.run_id.clone();
        self.core.revoke_run(&run_id);
    }
}

#[derive(Clone)]
struct HostGrantConfig {
    config: crate::mcp::McpServerConfig,
}

struct RelayInner {
    core: RelayCore,
    grant_configs: BTreeMap<String, HostGrantConfig>,
    pending_acks: BTreeMap<String, mpsc::Sender<RelayAckOutcome>>,
    endpoint: Option<PathBuf>,
    socket_dir: Option<PathBuf>,
    app_handle: Option<AppHandle>,
    active_connections: usize,
}

impl Default for RelayInner {
    fn default() -> Self {
        Self {
            core: RelayCore::default(),
            grant_configs: BTreeMap::new(),
            pending_acks: BTreeMap::new(),
            endpoint: None,
            socket_dir: None,
            app_handle: None,
            active_connections: 0,
        }
    }
}

/// Tauri-managed state.  All relay secrets and pending paths are bounded by
/// this process lifetime and are cleared on completion, revocation, timeout,
/// and app exit.
pub(crate) struct KnowledgeOpenRelayState {
    inner: Arc<Mutex<RelayInner>>,
    shutdown_requested: Arc<AtomicBool>,
}

impl KnowledgeOpenRelayState {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RelayInner::default())),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    #[cfg(unix)]
    pub(crate) fn start(&self, app_handle: AppHandle) -> Result<(), String> {
        let (socket_dir, endpoint) = create_relay_socket_path()?;
        let listener = UnixListener::bind(&endpoint)
            .map_err(|_| "knowledge_open_relay_listener_unavailable".to_string())?;
        std::fs::set_permissions(&endpoint, std::fs::Permissions::from_mode(0o600))
            .map_err(|_| "knowledge_open_relay_listener_unavailable".to_string())?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "knowledge_open_relay_listener_unavailable".to_string())?;

        {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| "knowledge_open_relay_state_unavailable".to_string())?;
            if inner.endpoint.is_some() {
                return Err("knowledge_open_relay_listener_already_started".to_string());
            }
            inner.endpoint = Some(endpoint);
            inner.socket_dir = Some(socket_dir);
            inner.app_handle = Some(app_handle);
        }

        let inner = Arc::clone(&self.inner);
        let shutdown_requested = Arc::clone(&self.shutdown_requested);
        std::thread::Builder::new()
            .name("knowledge-open-relay".to_string())
            .spawn(move || listener_loop(listener, inner, shutdown_requested))
            .map_err(|_| "knowledge_open_relay_listener_unavailable".to_string())?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub(crate) fn start(&self, _app_handle: AppHandle) -> Result<(), String> {
        Err("knowledge_open_relay_platform_unsupported".to_string())
    }

    pub(crate) fn issue_grant(
        &self,
        config: &crate::mcp::McpServerConfig,
        identity: RelayBindingIdentity,
    ) -> Result<KnowledgeOpenRelayMcpConfig, String> {
        if config.role != crate::mcp::McpRole::SupervisorOrchestrator
            || config.knowledge_open_relay.is_some()
            || config.run_id != identity.run_id
            || !valid_internal_identifier(&identity.turn_id)
            || !valid_internal_identifier(&identity.project_id)
        {
            return Err("knowledge_open_relay_grant_rejected".to_string());
        }
        let grant = random_hex(32)?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| "knowledge_open_relay_state_unavailable".to_string())?;
        cleanup_expired_locked(&mut inner, Instant::now());
        let endpoint = inner
            .endpoint
            .as_ref()
            .and_then(|path| path.to_str())
            .ok_or_else(|| "knowledge_open_relay_listener_unavailable".to_string())?
            .to_string();
        inner
            .core
            .issue(grant.clone(), identity.clone(), Instant::now())?;
        inner.grant_configs.insert(
            grant.clone(),
            HostGrantConfig {
                config: config.clone(),
            },
        );
        KnowledgeOpenRelayMcpConfig::from_mcp_arguments(
            endpoint,
            grant,
            identity.turn_id,
            identity.project_id,
        )
    }

    pub(crate) fn revoke_run(&self, run_id: &str) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        let cleanup = inner.core.revoke_run(run_id);
        for grant in cleanup.grants {
            inner.grant_configs.remove(&grant);
        }
        for intent_id in cleanup.intents {
            inner.pending_acks.remove(&intent_id);
        }
    }

    pub(crate) fn shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
        let (endpoint, socket_dir) = match self.inner.lock() {
            Ok(mut inner) => {
                inner.core = RelayCore::default();
                inner.grant_configs.clear();
                inner.pending_acks.clear();
                inner.app_handle = None;
                (inner.endpoint.take(), inner.socket_dir.take())
            }
            Err(_) => (None, None),
        };
        if let Some(endpoint) = endpoint {
            let _ = std::fs::remove_file(endpoint);
        }
        if let Some(socket_dir) = socket_dir {
            let _ = std::fs::remove_dir(socket_dir);
        }
    }

    fn acknowledge_ui(&self, request: KnowledgeOpenRelayAckRequest) -> Result<(), String> {
        request.validate()?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| "knowledge_open_relay_state_unavailable".to_string())?;
        cleanup_expired_locked(&mut inner, Instant::now());
        let snapshot = inner.core.acknowledge(
            &request.intent_id,
            &request.relative_path,
            request.outcome.as_core_outcome(),
            Instant::now(),
        )?;
        let sender = inner
            .pending_acks
            .remove(snapshot.intent_id())
            .ok_or_else(|| "knowledge_open_relay_ack_unavailable".to_string())?;
        sender
            .send(request.outcome.as_core_outcome())
            .map_err(|_| "knowledge_open_relay_ack_unavailable".to_string())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct KnowledgeOpenRelayAckRequest {
    intent_id: String,
    relative_path: String,
    outcome: KnowledgeOpenRelayAckWireOutcome,
}

impl KnowledgeOpenRelayAckRequest {
    fn validate(&self) -> Result<(), String> {
        if !valid_intent_id(&self.intent_id)
            || crate::mcp::supervisor_orchestrator::knowledge_capabilities::validate_knowledge_markdown_relative_path(
                &self.relative_path,
            )
            .is_err()
        {
            return Err("knowledge_open_relay_ack_rejected".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum KnowledgeOpenRelayAckWireOutcome {
    Opened,
    Rejected,
}

impl KnowledgeOpenRelayAckWireOutcome {
    fn as_core_outcome(self) -> RelayAckOutcome {
        match self {
            Self::Opened => RelayAckOutcome::Opened,
            Self::Rejected => RelayAckOutcome::Rejected,
        }
    }
}

#[tauri::command]
pub(crate) fn acknowledge_knowledge_open_relay_intent(
    request: KnowledgeOpenRelayAckRequest,
    window: tauri::WebviewWindow,
    relay: tauri::State<'_, KnowledgeOpenRelayState>,
) -> Result<(), String> {
    validate_main_window_ack_source(window.label())?;
    relay.acknowledge_ui(request)
}

/// A relay acknowledgement is only meaningful once the main Syn workspace has
/// handled the intent.  Other webviews must not be able to settle it.
pub(crate) fn validate_main_window_ack_source(window_label: &str) -> Result<(), String> {
    if window_label == "main" {
        Ok(())
    } else {
        Err("knowledge_open_relay_ack_source_rejected".to_string())
    }
}

#[derive(Clone, Serialize)]
struct KnowledgeOpenRelayUiIntent {
    schema_version: u8,
    intent_id: String,
    relative_path: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RelayWireIntent {
    schema_version: u8,
    kind: String,
    grant: String,
    run_id: String,
    turn_id: String,
    project_id: String,
    request_id: String,
    relative_path: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RelayWireResponse {
    schema_version: u8,
    kind: String,
    intent_id: String,
    relative_path: String,
    outcome: String,
}

pub(crate) struct KnowledgeOpenRelayOpened {
    pub(crate) intent_id: String,
    pub(crate) relative_path: String,
}

/// Called only from the fixed `knowledge_open` capability handler after the
/// existing fixed-vault validation.  A caller cannot provide a route, command,
/// endpoint, or grant through tool arguments.
pub(crate) fn dispatch_from_mcp(
    config: &crate::mcp::McpServerConfig,
    relative_path: &str,
) -> Result<KnowledgeOpenRelayOpened, String> {
    let relay_config = config
        .knowledge_open_relay
        .as_ref()
        .ok_or_else(|| "knowledge_open_relay_unavailable".to_string())?;
    if crate::mcp::supervisor_orchestrator::knowledge_capabilities::validate_knowledge_markdown_relative_path(relative_path)
        .is_err()
    {
        return Err("knowledge_open_relay_path_rejected".to_string());
    }
    let request = RelayWireIntent {
        schema_version: RELAY_SCHEMA_VERSION,
        kind: "knowledge_open_intent".to_string(),
        grant: relay_config.grant.clone(),
        run_id: config.run_id.clone(),
        turn_id: relay_config.turn_id.clone(),
        project_id: relay_config.project_id.clone(),
        request_id: format!("request:{}", random_hex(16)?),
        relative_path: relative_path.to_string(),
    };
    let payload =
        serde_json::to_vec(&request).map_err(|_| "knowledge_open_relay_unavailable".to_string())?;
    if payload.len() > MAX_RELAY_FRAME_BYTES {
        return Err("knowledge_open_relay_request_rejected".to_string());
    }

    #[cfg(unix)]
    {
        let mut stream = UnixStream::connect(&relay_config.endpoint)
            .map_err(|_| "knowledge_open_relay_unavailable".to_string())?;
        stream
            .set_read_timeout(Some(UI_ACK_TIMEOUT))
            .map_err(|_| "knowledge_open_relay_unavailable".to_string())?;
        stream
            .set_write_timeout(Some(UI_ACK_TIMEOUT))
            .map_err(|_| "knowledge_open_relay_unavailable".to_string())?;
        write_frame(&mut stream, &payload)?;
        let response = read_frame(&mut stream)?;
        let response: RelayWireResponse = serde_json::from_slice(&response)
            .map_err(|_| "knowledge_open_relay_response_rejected".to_string())?;
        if response.schema_version != RELAY_SCHEMA_VERSION
            || response.kind != "knowledge_open_ack"
            || response.outcome != "opened"
            || response.relative_path != relative_path
            || !valid_intent_id(&response.intent_id)
        {
            return Err("knowledge_open_relay_response_rejected".to_string());
        }
        Ok(KnowledgeOpenRelayOpened {
            intent_id: response.intent_id,
            relative_path: response.relative_path,
        })
    }
    #[cfg(not(unix))]
    {
        let _ = request;
        Err("knowledge_open_relay_platform_unsupported".to_string())
    }
}

#[cfg(unix)]
fn listener_loop(
    listener: UnixListener,
    inner: Arc<Mutex<RelayInner>>,
    shutdown_requested: Arc<AtomicBool>,
) {
    while !shutdown_requested.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let permitted = match inner.lock() {
                    Ok(mut state) if state.active_connections < MAX_ACTIVE_CONNECTIONS => {
                        state.active_connections += 1;
                        true
                    }
                    _ => false,
                };
                if !permitted {
                    continue;
                }
                let connection_inner = Arc::clone(&inner);
                std::thread::spawn(move || {
                    let _ = handle_connection(stream, &connection_inner);
                    if let Ok(mut state) = connection_inner.lock() {
                        state.active_connections = state.active_connections.saturating_sub(1);
                    }
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

#[cfg(unix)]
fn handle_connection(mut stream: UnixStream, inner: &Arc<Mutex<RelayInner>>) -> Result<(), String> {
    stream
        .set_read_timeout(Some(UI_ACK_TIMEOUT))
        .map_err(|_| "knowledge_open_relay_unavailable".to_string())?;
    stream
        .set_write_timeout(Some(UI_ACK_TIMEOUT))
        .map_err(|_| "knowledge_open_relay_unavailable".to_string())?;
    let request = read_frame(&mut stream)?;
    let request: RelayWireIntent = serde_json::from_slice(&request)
        .map_err(|_| "knowledge_open_relay_request_rejected".to_string())?;
    let response = dispatch_wire_intent(inner, request)?;
    let payload = serde_json::to_vec(&response)
        .map_err(|_| "knowledge_open_relay_response_rejected".to_string())?;
    write_frame(&mut stream, &payload)
}

fn dispatch_wire_intent(
    inner: &Arc<Mutex<RelayInner>>,
    request: RelayWireIntent,
) -> Result<RelayWireResponse, String> {
    validate_wire_intent(&request)?;
    let identity =
        RelayBindingIdentity::new(&request.run_id, &request.turn_id, &request.project_id);
    let (config, app_handle) = {
        let mut state = inner
            .lock()
            .map_err(|_| "knowledge_open_relay_state_unavailable".to_string())?;
        cleanup_expired_locked(&mut state, Instant::now());
        state
            .core
            .inspect(&request.grant, &identity, Instant::now())?;
        let config = state
            .grant_configs
            .get(&request.grant)
            .ok_or_else(|| "knowledge_open_relay_grant_unavailable".to_string())?
            .config
            .clone();
        let app_handle = state
            .app_handle
            .clone()
            .ok_or_else(|| "knowledge_open_relay_listener_unavailable".to_string())?;
        (config, app_handle)
    };
    if config.run_id != request.run_id || config.knowledge_open_relay.is_some() {
        return Err("knowledge_open_relay_grant_rejected".to_string());
    }

    let binding = crate::mcp::supervisor_orchestrator::active_supervisor_conversation_binding(
        &config,
        "knowledge_open",
    )?
    .ok_or_else(|| "knowledge_open_relay_binding_unavailable".to_string())?;
    if binding.run_id != request.run_id
        || binding.turn_id != request.turn_id
        || binding.project_id != request.project_id
    {
        return Err("knowledge_open_relay_binding_rejected".to_string());
    }
    // Re-read the current fixed-vault projection in the host before emitting an
    // intent.  The only UI data still remains the relative Markdown path.
    crate::knowledge_index::workspace_read_markdown_at(
        &crate::knowledge_vault::workspace_vault_root(),
        &request.relative_path,
    )
    .map_err(|_| "knowledge_open_relay_path_rejected".to_string())?;

    let intent_id = format!("intent:{}", random_hex(16)?);
    let (snapshot, receiver) = {
        let mut state = inner
            .lock()
            .map_err(|_| "knowledge_open_relay_state_unavailable".to_string())?;
        cleanup_expired_locked(&mut state, Instant::now());
        let snapshot = state.core.accept(
            &request.grant,
            &identity,
            request.relative_path.clone(),
            intent_id,
            Instant::now(),
        )?;
        state.grant_configs.remove(&request.grant);
        let (sender, receiver) = mpsc::channel();
        state
            .pending_acks
            .insert(snapshot.intent_id().to_string(), sender);
        (snapshot, receiver)
    };

    let emit_result = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "knowledge_open_relay_window_unavailable".to_string())
        .and_then(|window| {
            window
                .show()
                .map_err(|_| "knowledge_open_relay_window_unavailable".to_string())?;
            window
                .set_focus()
                .map_err(|_| "knowledge_open_relay_window_unavailable".to_string())?;
            window
                .emit(
                    RELAY_EVENT_NAME,
                    KnowledgeOpenRelayUiIntent {
                        schema_version: RELAY_SCHEMA_VERSION,
                        intent_id: snapshot.intent_id().to_string(),
                        relative_path: request.relative_path.clone(),
                    },
                )
                .map_err(|_| "knowledge_open_relay_emit_failed".to_string())
        });
    if emit_result.is_err() {
        cancel_pending(inner, snapshot.intent_id());
        return Err("knowledge_open_relay_dispatch_unavailable".to_string());
    }

    match receiver.recv_timeout(UI_ACK_TIMEOUT) {
        Ok(RelayAckOutcome::Opened) => Ok(RelayWireResponse {
            schema_version: RELAY_SCHEMA_VERSION,
            kind: "knowledge_open_ack".to_string(),
            intent_id: snapshot.intent_id().to_string(),
            relative_path: request.relative_path,
            outcome: "opened".to_string(),
        }),
        Ok(RelayAckOutcome::Rejected) => Err("knowledge_open_relay_ui_rejected".to_string()),
        Err(_) => {
            cancel_pending(inner, snapshot.intent_id());
            Err("knowledge_open_relay_ack_timeout".to_string())
        }
    }
}

fn cancel_pending(inner: &Arc<Mutex<RelayInner>>, intent_id: &str) {
    if let Ok(mut state) = inner.lock() {
        state.core.cancel(intent_id);
        state.pending_acks.remove(intent_id);
    }
}

fn cleanup_expired_locked(inner: &mut RelayInner, now: Instant) {
    let cleanup = inner.core.expire(now);
    for grant in cleanup.grants {
        inner.grant_configs.remove(&grant);
    }
    for intent_id in cleanup.intents {
        inner.pending_acks.remove(&intent_id);
    }
}

fn validate_wire_intent(request: &RelayWireIntent) -> Result<(), String> {
    if request.schema_version != RELAY_SCHEMA_VERSION
        || request.kind != "knowledge_open_intent"
        || !valid_secret_token(&request.grant)
        || !valid_internal_identifier(&request.run_id)
        || !valid_internal_identifier(&request.turn_id)
        || !valid_internal_identifier(&request.project_id)
        || !valid_request_id(&request.request_id)
        || crate::mcp::supervisor_orchestrator::knowledge_capabilities::validate_knowledge_markdown_relative_path(
            &request.relative_path,
        )
        .is_err()
    {
        return Err("knowledge_open_relay_request_rejected".to_string());
    }
    Ok(())
}

#[cfg(unix)]
fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut header = [0_u8; 4];
    stream
        .read_exact(&mut header)
        .map_err(|_| "knowledge_open_relay_frame_unavailable".to_string())?;
    let frame_len = u32::from_be_bytes(header) as usize;
    if frame_len == 0 || frame_len > MAX_RELAY_FRAME_BYTES {
        return Err("knowledge_open_relay_frame_rejected".to_string());
    }
    let mut payload = vec![0_u8; frame_len];
    stream
        .read_exact(&mut payload)
        .map_err(|_| "knowledge_open_relay_frame_unavailable".to_string())?;
    Ok(payload)
}

#[cfg(unix)]
fn write_frame(stream: &mut UnixStream, payload: &[u8]) -> Result<(), String> {
    if payload.is_empty() || payload.len() > MAX_RELAY_FRAME_BYTES {
        return Err("knowledge_open_relay_frame_rejected".to_string());
    }
    stream
        .write_all(&(payload.len() as u32).to_be_bytes())
        .and_then(|_| stream.write_all(payload))
        .and_then(|_| stream.flush())
        .map_err(|_| "knowledge_open_relay_frame_unavailable".to_string())
}

#[cfg(unix)]
fn create_relay_socket_path() -> Result<(PathBuf, PathBuf), String> {
    for _ in 0..4 {
        let directory =
            std::env::temp_dir().join(format!("syn-knowledge-open-{}", random_hex(12)?));
        if std::fs::create_dir(&directory).is_err() {
            continue;
        }
        if std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).is_err() {
            let _ = std::fs::remove_dir(&directory);
            return Err("knowledge_open_relay_listener_unavailable".to_string());
        }
        return Ok((directory.clone(), directory.join("relay.sock")));
    }
    Err("knowledge_open_relay_listener_unavailable".to_string())
}

fn random_hex(byte_count: usize) -> Result<String, String> {
    let mut bytes = vec![0_u8; byte_count];
    getrandom::getrandom(&mut bytes)
        .map_err(|_| "knowledge_open_relay_random_unavailable".to_string())?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn valid_absolute_socket_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && Path::new(value).is_absolute()
        && !value.chars().any(char::is_control)
}

fn valid_secret_token(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_internal_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_'))
}

fn valid_request_id(value: &str) -> bool {
    value.starts_with("request:") && valid_internal_identifier(value)
}

fn valid_intent_id(value: &str) -> bool {
    value.starts_with("intent:") && valid_internal_identifier(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_wire_payload_rejects_extra_fields_and_invalid_values() {
        let valid = serde_json::json!({
            "schema_version": RELAY_SCHEMA_VERSION,
            "kind": "knowledge_open_intent",
            "grant": "a".repeat(64),
            "run_id": "supervisor-conversation:relay-wire-test",
            "turn_id": "turn:relay-wire-test",
            "project_id": "project:relay-wire-test",
            "request_id": "request:relay-wire-test",
            "relative_path": "research/OpenMe.md",
        });
        let request: RelayWireIntent =
            serde_json::from_value(valid.clone()).expect("exact wire request parses");
        assert!(validate_wire_intent(&request).is_ok());

        let mut extra_field = valid.clone();
        extra_field["route"] = serde_json::json!("knowledge");
        assert!(
            serde_json::from_value::<RelayWireIntent>(extra_field).is_err(),
            "the relay request schema rejects route-like extra fields"
        );

        let mut invalid_path = valid.clone();
        invalid_path["relative_path"] = serde_json::json!("research/OpenMe.canvas");
        let invalid_path: RelayWireIntent =
            serde_json::from_value(invalid_path).expect("schema still parses a string path");
        assert!(
            validate_wire_intent(&invalid_path).is_err(),
            "the wire path must remain an exact fixed-vault Markdown path"
        );

        let ack_with_extra = serde_json::json!({
            "intent_id": "intent:relay-wire-test",
            "relative_path": "research/OpenMe.md",
            "outcome": "opened",
            "command": "open",
        });
        assert!(
            serde_json::from_value::<KnowledgeOpenRelayAckRequest>(ack_with_extra).is_err(),
            "the UI acknowledgement cannot carry a route or command selector"
        );
    }
}
