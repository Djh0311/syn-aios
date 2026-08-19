// L3 可选 Obsidian 兼容层：固定官方 App/CLI/URI bridge。
// Syn 原生知识工作区不依赖它；前端只能选择已枚举的动作，binary、vault、argv 和
// macOS `open` 均由宿主固定。

use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const OFFICIAL_APP_BUNDLE: &str = "/Applications/Obsidian.app";
const OFFICIAL_APP_CLI_RELATIVE: &str = "Contents/MacOS/obsidian-cli";
const OFFICIAL_REGISTERED_CLI: &str = "/usr/local/bin/obsidian";
const PROCESS_LOOKUP: &str = "/usr/bin/pgrep";
const MACOS_OPEN: &str = "/usr/bin/open";
const MINIMUM_CLI_VERSION: (u16, u16, u16) = (1, 12, 7);
const CLI_TIMEOUT: Duration = Duration::from_secs(4);
const CLI_OUTPUT_LIMIT_BYTES: usize = 64 * 1024;
const SEARCH_QUERY_LIMIT: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObsidianReadiness {
    NotInstalled,
    Installed,
    AppNotRunning,
    CliNotEnabled,
    Ready,
    Incompatible,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ObsidianIntegrationStatus {
    status: ObsidianReadiness,
    message: String,
    app_version: Option<String>,
    cli_version: Option<String>,
    vault_label: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ObsidianActionReceipt {
    action: &'static str,
    message: String,
    degraded: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ObsidianTextResult {
    text: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ObsidianIntegrationNote {
    slug: String,
    title: String,
    body: String,
    mtime_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ObsidianIntegrationSearchResult {
    slug: String,
    title: String,
    snippet: String,
    mtime_ms: i64,
}

#[derive(Clone, Debug)]
struct BridgePaths {
    app_bundle: PathBuf,
    bundled_cli: PathBuf,
    registered_cli: PathBuf,
    vault_root: PathBuf,
}

impl BridgePaths {
    fn production() -> Self {
        let app_bundle = PathBuf::from(OFFICIAL_APP_BUNDLE);
        Self {
            bundled_cli: app_bundle.join(OFFICIAL_APP_CLI_RELATIVE),
            registered_cli: PathBuf::from(OFFICIAL_REGISTERED_CLI),
            vault_root: crate::knowledge_vault::syn_vault_root(),
            app_bundle,
        }
    }
}

#[derive(Clone, Debug)]
struct CliOutput {
    stdout: String,
    _stderr: String,
}

trait ObsidianProbe {
    fn app_bundle_exists(&self, paths: &BridgePaths) -> bool;
    fn bundled_cli_exists(&self, paths: &BridgePaths) -> bool;
    fn registered_cli_matches_bundle(&self, paths: &BridgePaths) -> bool;
    fn app_is_running(&self) -> bool;
    fn run_cli(&self, paths: &BridgePaths, args: &[String]) -> Result<CliOutput, String>;
}

struct SystemObsidianProbe;

impl ObsidianProbe for SystemObsidianProbe {
    fn app_bundle_exists(&self, paths: &BridgePaths) -> bool {
        paths.app_bundle.is_dir()
    }

    fn bundled_cli_exists(&self, paths: &BridgePaths) -> bool {
        paths.bundled_cli.is_file()
    }

    fn registered_cli_matches_bundle(&self, paths: &BridgePaths) -> bool {
        let Ok(registered) = fs::canonicalize(&paths.registered_cli) else {
            return false;
        };
        let Ok(bundled) = fs::canonicalize(&paths.bundled_cli) else {
            return false;
        };
        registered == bundled
    }

    fn app_is_running(&self) -> bool {
        Command::new(PROCESS_LOOKUP)
            .args(["-x", "Obsidian"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn run_cli(&self, paths: &BridgePaths, args: &[String]) -> Result<CliOutput, String> {
        run_process_with_limits(
            &paths.registered_cli,
            &paths.vault_root,
            args,
            CLI_TIMEOUT,
            CLI_OUTPUT_LIMIT_BYTES,
        )
    }
}

fn status_with(paths: &BridgePaths, probe: &impl ObsidianProbe) -> ObsidianIntegrationStatus {
    let base = |status, message| ObsidianIntegrationStatus {
        status,
        message,
        app_version: None,
        cli_version: None,
        vault_label: "Syn 自管 Markdown vault",
    };

    if !probe.app_bundle_exists(paths) {
        return base(
            ObsidianReadiness::NotInstalled,
            "未检测到 /Applications 中的官方 Obsidian。Syn 原生 Markdown 页面仍可用。".to_string(),
        );
    }
    if !probe.bundled_cli_exists(paths) {
        return base(
            ObsidianReadiness::Installed,
            "检测到官方 Obsidian，但未找到其受支持的 CLI binary。".to_string(),
        );
    }
    if !probe.registered_cli_matches_bundle(paths) {
        return base(
            ObsidianReadiness::CliNotEnabled,
            "Obsidian CLI 尚未由官方设置注册；Syn 不会自行修改 PATH 或创建链接。".to_string(),
        );
    }
    if !probe.app_is_running() {
        return base(
            ObsidianReadiness::AppNotRunning,
            "Obsidian CLI 已注册，但官方 App 当前未运行。".to_string(),
        );
    }

    let version_args = vec!["version".to_string()];
    let output = match probe.run_cli(paths, &version_args) {
        Ok(output) => output,
        Err(_) => {
            return base(
                ObsidianReadiness::Incompatible,
                "官方 Obsidian 已运行，但 CLI version 探测失败；Syn 保持原生 Markdown 降级。"
                    .to_string(),
            )
        }
    };
    let version = match parse_version(&output.stdout) {
        Some(version) => version,
        None => {
            return base(
                ObsidianReadiness::Incompatible,
                "官方 Obsidian CLI 未返回可识别版本；Syn 不会猜测其兼容性。".to_string(),
            )
        }
    };
    if version_tuple(&version) < MINIMUM_CLI_VERSION {
        return ObsidianIntegrationStatus {
            status: ObsidianReadiness::Incompatible,
            message: format!("Obsidian {version} 低于官方 CLI 所需的 1.12.7。"),
            app_version: Some(version.clone()),
            cli_version: Some(version),
            vault_label: "Syn 自管 Markdown vault",
        };
    }

    ObsidianIntegrationStatus {
        status: ObsidianReadiness::Ready,
        message: "官方 Obsidian、CLI 和 Syn 自管 vault 的 typed bridge 已就绪。".to_string(),
        app_version: Some(version.clone()),
        cli_version: Some(version),
        vault_label: "Syn 自管 Markdown vault",
    }
}

fn require_ready_with(paths: &BridgePaths, probe: &impl ObsidianProbe) -> Result<(), String> {
    let status = status_with(paths, probe);
    if status.status == ObsidianReadiness::Ready {
        return Ok(());
    }
    Err(readiness_error(&status))
}

fn integration_error(code: &str, message: &str) -> String {
    format!("{code}: {message}")
}

fn readiness_error(status: &ObsidianIntegrationStatus) -> String {
    let code = match status.status {
        ObsidianReadiness::NotInstalled | ObsidianReadiness::Installed => {
            "obsidian_integration_not_installed"
        }
        ObsidianReadiness::AppNotRunning => "obsidian_integration_app_not_running",
        ObsidianReadiness::CliNotEnabled => "obsidian_integration_cli_not_enabled",
        ObsidianReadiness::Ready => "obsidian_integration_ready",
        ObsidianReadiness::Incompatible => "obsidian_integration_incompatible",
    };
    integration_error(code, &status.message)
}

fn parse_version(text: &str) -> Option<String> {
    let token = text
        .split(|character: char| !(character.is_ascii_digit() || character == '.'))
        .find(|token| token.matches('.').count() >= 2)?;
    let parsed = version_tuple(token);
    if parsed == (0, 0, 0) {
        None
    } else {
        Some(token.to_string())
    }
}

fn version_tuple(text: &str) -> (u16, u16, u16) {
    let mut parts = text.split('.').filter_map(|part| part.parse::<u16>().ok());
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

fn validate_query(query: &str) -> Result<String, String> {
    let query = query.trim();
    if query.is_empty() || query.len() > SEARCH_QUERY_LIMIT {
        return Err(integration_error(
            "obsidian_integration_invalid_query",
            "搜索文本不能为空且不能超过 256 个字节。",
        ));
    }
    if query.starts_with('-')
        || query.starts_with('/')
        || query.contains(['\n', '\r', '\0', '\'', '"', '\\'])
        || query.contains("--")
    {
        return Err(integration_error(
            "obsidian_integration_invalid_query",
            "搜索文本包含不允许的控制参数。",
        ));
    }
    Ok(query.to_string())
}

fn cli_args(command: TypedCliCommand) -> Result<Vec<String>, String> {
    match command {
        TypedCliCommand::Read { slug } => {
            let path = crate::knowledge_vault::syn_note_relative_path(&slug).map_err(|_| {
                integration_error(
                    "obsidian_integration_invalid_slug",
                    "笔记定位不符合 Syn 自管 vault 边界。",
                )
            })?;
            Ok(vec!["read".to_string(), format!("path={path}")])
        }
        TypedCliCommand::Search { query } => Ok(vec![
            "search".to_string(),
            format!("query={}", validate_query(&query)?),
            "limit=20".to_string(),
            "format=json".to_string(),
        ]),
        TypedCliCommand::OpenSearch { query } => Ok(vec![
            "search:open".to_string(),
            format!("query={}", validate_query(&query)?),
        ]),
    }
}

#[derive(Clone, Debug)]
enum TypedCliCommand {
    Read { slug: String },
    Search { query: String },
    OpenSearch { query: String },
}

fn execute_cli_with(
    paths: &BridgePaths,
    probe: &impl ObsidianProbe,
    command: TypedCliCommand,
) -> Result<ObsidianTextResult, String> {
    require_ready_with(paths, probe)?;
    let args = cli_args(command)?;
    let output = probe.run_cli(paths, &args)?;
    Ok(ObsidianTextResult {
        text: output.stdout,
    })
}

fn execute_production_cli(command: TypedCliCommand) -> Result<ObsidianTextResult, String> {
    let mut paths = BridgePaths::production();
    paths.vault_root = safe_external_vault_root()?;
    execute_cli_with(&paths, &SystemObsidianProbe, command)
}

fn safe_external_vault_root() -> Result<PathBuf, String> {
    crate::knowledge_vault::syn_vault_root_for_external_integration().map_err(|_| {
        integration_error(
            "obsidian_integration_vault_unavailable",
            "Syn 自管 vault 尚不可安全交给 Obsidian。",
        )
    })
}

fn percent_encode_query_value(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn vault_open_uri(vault_root: &Path) -> String {
    format!(
        "obsidian://open?path={}",
        percent_encode_query_value(&vault_root.display().to_string())
    )
}

fn note_open_uri(vault_root: &Path, slug: &str) -> Result<String, String> {
    let relative_path = crate::knowledge_vault::syn_note_relative_path(slug).map_err(|_| {
        integration_error(
            "obsidian_integration_invalid_slug",
            "笔记定位不符合 Syn 自管 vault 边界。",
        )
    })?;
    Ok(format!(
        "obsidian://open?path={}",
        percent_encode_query_value(&vault_root.join(relative_path).display().to_string())
    ))
}

fn open_uri(uri: &str) -> Result<(), String> {
    let status = Command::new(MACOS_OPEN)
        .args(["-a", OFFICIAL_APP_BUNDLE, uri])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| {
            integration_error(
                "obsidian_integration_open_failed",
                "无法调用 macOS 的官方 URI 打开器。",
            )
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(integration_error(
            "obsidian_integration_open_failed",
            "Obsidian URI 打开失败；Syn 保持原生 Markdown 降级。",
        ))
    }
}

fn action_receipt(action: &'static str, message: &str, degraded: bool) -> ObsidianActionReceipt {
    ObsidianActionReceipt {
        action,
        message: message.to_string(),
        degraded,
    }
}

#[tauri::command]
pub(crate) fn obsidian_integration_status() -> ObsidianIntegrationStatus {
    status_with(&BridgePaths::production(), &SystemObsidianProbe)
}

#[tauri::command]
pub(crate) fn obsidian_integration_open_vault() -> Result<ObsidianActionReceipt, String> {
    let paths = BridgePaths::production();
    if !paths.app_bundle.is_dir() {
        return Err(integration_error(
            "obsidian_integration_not_installed",
            "未安装官方 Obsidian，无法打开 Syn 自管 vault。",
        ));
    }
    let vault_root = safe_external_vault_root()?;
    open_uri(&vault_open_uri(&vault_root))?;
    Ok(action_receipt(
        "open_vault",
        "已请求用官方 Obsidian 打开 Syn 自管 vault。",
        false,
    ))
}

#[tauri::command]
pub(crate) fn obsidian_integration_open_note(
    slug: String,
) -> Result<ObsidianActionReceipt, String> {
    open_syn_note_from_supervisor(&slug)?;
    Ok(action_receipt(
        "open_note",
        "已请求用官方 Obsidian 打开该 Syn 笔记。",
        false,
    ))
}

// 主管 capability `knowledge_open` 只复用同一个固定 URI 路径；它不能传入 app、vault 或 argv。
pub(crate) fn open_syn_note_from_supervisor(slug: &str) -> Result<(), String> {
    let paths = BridgePaths::production();
    if !paths.app_bundle.is_dir() {
        return Err(integration_error(
            "obsidian_integration_not_installed",
            "未安装官方 Obsidian，无法打开 Syn 笔记。",
        ));
    }
    let vault_root = safe_external_vault_root()?;
    open_uri(&note_open_uri(&vault_root, slug)?)
}

#[tauri::command]
pub(crate) fn obsidian_integration_open_search(
    query: String,
) -> Result<ObsidianActionReceipt, String> {
    execute_production_cli(TypedCliCommand::OpenSearch { query })?;
    Ok(action_receipt(
        "open_search",
        "已在官方 Obsidian 中打开 Syn vault 搜索。",
        false,
    ))
}

fn title_from_markdown_or_slug(slug: &str, body: &str) -> String {
    body.lines()
        .find_map(|line| line.trim_start().strip_prefix("# "))
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| slug.to_string())
}

fn safe_note_mtime_ms(slug: &str) -> Result<i64, String> {
    let vault_root = safe_external_vault_root()?;
    let relative_path = crate::knowledge_vault::syn_note_relative_path(slug).map_err(|_| {
        integration_error(
            "obsidian_integration_invalid_slug",
            "笔记定位不符合 Syn 自管 vault 边界。",
        )
    })?;
    let path = vault_root.join(relative_path);
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        integration_error(
            "obsidian_integration_note_unavailable",
            "Syn 自管 vault 中不存在该笔记。",
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(integration_error(
            "obsidian_integration_note_unavailable",
            "该笔记不是可安全读取的 Syn Markdown 文件。",
        ));
    }
    Ok(metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0))
}

fn search_slug_from_cli_path(path: &str) -> Option<String> {
    let path = Path::new(path);
    if path.components().count() != 1
        || path.extension().and_then(|extension| extension.to_str()) != Some("md")
    {
        return None;
    }
    let slug = path.file_stem()?.to_str()?;
    if crate::knowledge_vault::syn_note_relative_path(slug)
        .ok()
        .as_deref()
        == Some(path.file_name()?.to_str()?)
    {
        Some(slug.to_string())
    } else {
        None
    }
}

fn parse_search_results(output: &str) -> Result<Vec<ObsidianIntegrationSearchResult>, String> {
    let paths: Vec<String> = serde_json::from_str(output).map_err(|_| {
        integration_error(
            "obsidian_integration_invalid_output",
            "Obsidian 搜索未返回受支持的受限 JSON 结果。",
        )
    })?;
    let mut results = Vec::new();
    for path in paths {
        let Some(slug) = search_slug_from_cli_path(&path) else {
            continue;
        };
        let mtime_ms = safe_note_mtime_ms(&slug)?;
        results.push(ObsidianIntegrationSearchResult {
            title: slug.clone(),
            slug,
            snippet: "Obsidian 搜索命中该 Syn 笔记。".to_string(),
            mtime_ms,
        });
    }
    Ok(results)
}

#[tauri::command]
pub(crate) fn obsidian_integration_read_note(
    slug: String,
) -> Result<ObsidianIntegrationNote, String> {
    let output = execute_production_cli(TypedCliCommand::Read { slug: slug.clone() })?;
    Ok(ObsidianIntegrationNote {
        title: title_from_markdown_or_slug(&slug, &output.text),
        mtime_ms: safe_note_mtime_ms(&slug)?,
        slug,
        body: output.text,
    })
}

#[tauri::command]
pub(crate) fn obsidian_integration_search_notes(
    query: String,
) -> Result<Vec<ObsidianIntegrationSearchResult>, String> {
    parse_search_results(&execute_production_cli(TypedCliCommand::Search { query })?.text)
}

fn run_process_with_limits(
    executable: &Path,
    cwd: &Path,
    args: &[String],
    timeout: Duration,
    output_limit: usize,
) -> Result<CliOutput, String> {
    let mut child = Command::new(executable)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| {
            integration_error(
                "obsidian_integration_launch_failed",
                "无法启动固定官方 Obsidian CLI。",
            )
        })?;

    let stdout = child.stdout.take().ok_or_else(|| {
        integration_error(
            "obsidian_integration_output_capture_failed",
            "无法捕获 Obsidian CLI stdout。",
        )
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        integration_error(
            "obsidian_integration_output_capture_failed",
            "无法捕获 Obsidian CLI stderr。",
        )
    })?;
    let (sender, receiver) = mpsc::channel();
    spawn_capped_reader(sender.clone(), StreamKind::Stdout, stdout, output_limit);
    spawn_capped_reader(sender, StreamKind::Stderr, stderr, output_limit);

    let deadline = Instant::now() + timeout;
    let mut stdout = None;
    let mut stderr = None;
    let mut exit_status = None;
    while exit_status.is_none() {
        while let Ok(stream) = receiver.try_recv() {
            if stream.exceeded {
                let _ = child.kill();
                let _ = child.wait();
                return Err(integration_error(
                    "obsidian_integration_output_limit",
                    "Obsidian CLI 输出超过安全上限，已终止。",
                ));
            }
            match stream.kind {
                StreamKind::Stdout => stdout = Some(stream.bytes),
                StreamKind::Stderr => stderr = Some(stream.bytes),
            }
        }
        exit_status = child.try_wait().map_err(|_| {
            integration_error(
                "obsidian_integration_execution_failed",
                "无法检查 Obsidian CLI 执行状态。",
            )
        })?;
        if exit_status.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(integration_error(
                "obsidian_integration_timeout",
                "Obsidian CLI 超时，已终止。",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    let exit_status = exit_status.expect("child exits before output conversion");
    while stdout.is_none() || stderr.is_none() {
        let stream = receiver
            .recv_timeout(Duration::from_millis(250))
            .map_err(|_| {
                integration_error(
                    "obsidian_integration_output_capture_failed",
                    "无法完整捕获 Obsidian CLI 输出。",
                )
            })?;
        if stream.exceeded {
            return Err(integration_error(
                "obsidian_integration_output_limit",
                "Obsidian CLI 输出超过安全上限，已终止。",
            ));
        }
        match stream.kind {
            StreamKind::Stdout => stdout = Some(stream.bytes),
            StreamKind::Stderr => stderr = Some(stream.bytes),
        }
    }
    if !exit_status.success() {
        return Err(integration_error(
            "obsidian_integration_command_failed",
            &format!(
                "Obsidian CLI 执行失败（退出码 {}）。",
                exit_status.code().unwrap_or(-1)
            ),
        ));
    }
    let stdout = String::from_utf8(stdout.expect("stdout set")).map_err(|_| {
        integration_error(
            "obsidian_integration_invalid_output",
            "Obsidian CLI stdout 不是有效 UTF-8。",
        )
    })?;
    let stderr = String::from_utf8(stderr.expect("stderr set")).map_err(|_| {
        integration_error(
            "obsidian_integration_invalid_output",
            "Obsidian CLI stderr 不是有效 UTF-8。",
        )
    })?;
    Ok(CliOutput {
        stdout,
        _stderr: stderr,
    })
}

#[derive(Clone, Copy)]
enum StreamKind {
    Stdout,
    Stderr,
}

struct StreamCapture {
    kind: StreamKind,
    bytes: Vec<u8>,
    exceeded: bool,
}

fn spawn_capped_reader<R: Read + Send + 'static>(
    sender: mpsc::Sender<StreamCapture>,
    kind: StreamKind,
    mut reader: R,
    output_limit: usize,
) {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        let mut exceeded = false;
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if bytes.len().saturating_add(count) > output_limit {
                        exceeded = true;
                        break;
                    }
                    bytes.extend_from_slice(&buffer[..count]);
                }
                Err(_) => break,
            }
        }
        let _ = sender.send(StreamCapture {
            kind,
            bytes,
            exceeded,
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static FAKE_EXECUTABLE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    struct FakeProbe {
        app_exists: bool,
        bundled_cli_exists: bool,
        cli_registered: bool,
        app_running: bool,
        cli_result: Result<CliOutput, String>,
        invocations: RefCell<Vec<Vec<String>>>,
    }

    impl Default for FakeProbe {
        fn default() -> Self {
            Self {
                app_exists: false,
                bundled_cli_exists: false,
                cli_registered: false,
                app_running: false,
                cli_result: Err("fake CLI 未配置".to_string()),
                invocations: RefCell::new(Vec::new()),
            }
        }
    }

    impl FakeProbe {
        fn ready(version: &str) -> Self {
            Self {
                app_exists: true,
                bundled_cli_exists: true,
                cli_registered: true,
                app_running: true,
                cli_result: Ok(CliOutput {
                    stdout: version.to_string(),
                    _stderr: String::new(),
                }),
                invocations: RefCell::new(Vec::new()),
            }
        }
    }

    impl ObsidianProbe for FakeProbe {
        fn app_bundle_exists(&self, _paths: &BridgePaths) -> bool {
            self.app_exists
        }

        fn bundled_cli_exists(&self, _paths: &BridgePaths) -> bool {
            self.bundled_cli_exists
        }

        fn registered_cli_matches_bundle(&self, _paths: &BridgePaths) -> bool {
            self.cli_registered
        }

        fn app_is_running(&self) -> bool {
            self.app_running
        }

        fn run_cli(&self, _paths: &BridgePaths, args: &[String]) -> Result<CliOutput, String> {
            self.invocations.borrow_mut().push(args.to_vec());
            self.cli_result.clone()
        }
    }

    fn fixture_paths() -> BridgePaths {
        let root = std::env::temp_dir().join("obsidian-integration-fixture-vault");
        BridgePaths {
            app_bundle: PathBuf::from("/fixed/Obsidian.app"),
            bundled_cli: PathBuf::from("/fixed/Obsidian.app/Contents/MacOS/obsidian-cli"),
            registered_cli: PathBuf::from("/fixed/obsidian"),
            vault_root: root,
        }
    }

    #[test]
    fn readiness_covers_all_six_states() {
        let paths = fixture_paths();
        let not_installed = FakeProbe::default();
        assert_eq!(
            status_with(&paths, &not_installed).status,
            ObsidianReadiness::NotInstalled
        );

        let installed = FakeProbe {
            app_exists: true,
            ..Default::default()
        };
        assert_eq!(
            status_with(&paths, &installed).status,
            ObsidianReadiness::Installed
        );

        let cli_not_enabled = FakeProbe {
            app_exists: true,
            bundled_cli_exists: true,
            ..Default::default()
        };
        assert_eq!(
            status_with(&paths, &cli_not_enabled).status,
            ObsidianReadiness::CliNotEnabled
        );

        let app_not_running = FakeProbe {
            app_exists: true,
            bundled_cli_exists: true,
            cli_registered: true,
            ..Default::default()
        };
        assert_eq!(
            status_with(&paths, &app_not_running).status,
            ObsidianReadiness::AppNotRunning
        );

        let incompatible = FakeProbe::ready("Obsidian 1.12.6");
        assert_eq!(
            status_with(&paths, &incompatible).status,
            ObsidianReadiness::Incompatible
        );

        let ready = FakeProbe::ready("Obsidian 1.12.7");
        assert_eq!(status_with(&paths, &ready).status, ObsidianReadiness::Ready);
    }

    #[test]
    fn optional_compatibility_status_is_nonblocking_and_minimal() {
        let status = status_with(&fixture_paths(), &FakeProbe::default());
        let serialized = serde_json::to_value(&status).expect("status should serialize");

        assert_eq!(serialized["status"], "not_installed");
        let mut fields = serialized
            .as_object()
            .expect("serialized status should be an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        fields.sort_unstable();
        assert_eq!(
            fields,
            vec![
                "app_version",
                "cli_version",
                "message",
                "status",
                "vault_label"
            ],
            "N0 compatibility status must expose only its minimal typed availability fields"
        );
        assert!(
            status.message.contains("Syn 原生 Markdown"),
            "missing Obsidian must remain a non-blocking compatibility result"
        );
    }

    #[test]
    fn optional_compatibility_surface_exposes_no_external_control_handlers() {
        for source in [
            include_str!("obsidian_integration.rs"),
            include_str!("command_registry.rs"),
        ] {
            for stopped_handler in [
                ["obsidian_integration_list", "_commands"].concat(),
                ["obsidian_integration_run", "_allowed_command"].concat(),
                ["obsidian_integration_re", "start"].concat(),
            ] {
                assert!(
                    !source.contains(&stopped_handler),
                    "optional compatibility bridge must not expose {stopped_handler}"
                );
            }
        }
    }

    #[test]
    fn typed_cli_uses_fixed_argv_and_never_accepts_a_program_or_vault() {
        let paths = fixture_paths();
        let probe = FakeProbe::ready("1.12.7");
        let result = execute_cli_with(
            &paths,
            &probe,
            TypedCliCommand::Read {
                slug: "meeting-note".to_string(),
            },
        )
        .expect("fake CLI succeeds");
        assert_eq!(result.text, "1.12.7");
        let invocations = probe.invocations.borrow();
        assert_eq!(invocations[0], vec!["version"]);
        assert_eq!(
            invocations[1],
            vec!["read", "path=meeting-note.md"],
            "only typed subcommand and fixed relative path are allowed"
        );
    }

    #[test]
    fn reject_path_option_and_query_injection_forms() {
        for slug in [
            "../escape",
            "/etc/passwd",
            "--help",
            "line\nbreak",
            "quote\"note",
            "wild*card",
        ] {
            assert!(
                crate::knowledge_vault::syn_note_relative_path(slug).is_err(),
                "must reject {slug:?}"
            );
        }
        for query in [
            "--help",
            "-flag",
            "/absolute",
            "hello\nworld",
            "quote\"query",
            "x -- y",
        ] {
            assert!(validate_query(query).is_err(), "must reject {query:?}");
        }
    }

    #[test]
    fn uri_builder_encodes_the_fixed_vault_path() {
        let vault_root = PathBuf::from("/tmp/Syn Vault & Notes");
        assert_eq!(
            vault_open_uri(&vault_root),
            "obsidian://open?path=%2Ftmp%2FSyn%20Vault%20%26%20Notes"
        );
    }

    #[cfg(unix)]
    fn fake_cli_cwd(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "obsidian-integration-cli-test-{}-{}-{}",
            std::process::id(),
            tag,
            FAKE_EXECUTABLE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    // 进程夹具确定性边界：不新建脚本文件，直接把载荷作为 argv 喂给系统 /bin/sh。
    // 新建脚本首次 exec 在本沙箱实测 155ms~3.2s（系统对新可执行文件的检查），
    // 全量并行时稳定撞穿 1s deadline；/bin/sh 本身常驻温热（实测 ~7ms），
    // exec 延迟不再是夹具变量，产品超时/上限语义一字不动。
    #[cfg(unix)]
    fn fake_sh_argv(body: &str) -> Vec<String> {
        vec!["-c".to_string(), body.to_string()]
    }

    #[cfg(unix)]
    #[test]
    fn fake_executable_proves_nonzero_timeout_and_output_cap_are_closed() {
        let sh = Path::new("/bin/sh");

        let root = fake_cli_cwd("nonzero");
        let error = run_process_with_limits(
            sh,
            &root,
            &fake_sh_argv("exit 7"),
            Duration::from_secs(1),
            1024,
        )
        .unwrap_err();
        assert!(
            error.contains("退出码 7"),
            "nonzero fake CLI must retain the sanitized exit-status reason, got: {error}"
        );
        let _ = fs::remove_dir_all(&root);

        let root = fake_cli_cwd("timeout");
        let error = run_process_with_limits(
            sh,
            &root,
            &fake_sh_argv("sleep 1"),
            Duration::from_millis(20),
            1024,
        )
        .unwrap_err();
        assert!(error.contains("超时"));
        let _ = fs::remove_dir_all(&root);

        let root = fake_cli_cwd("too-large");
        let error = run_process_with_limits(
            sh,
            &root,
            &fake_sh_argv("yes x | head -c 2048"),
            Duration::from_secs(1),
            128,
        )
        .unwrap_err();
        assert!(error.contains("超过安全上限"));
        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn fake_executable_proves_non_utf8_is_rejected() {
        let root = fake_cli_cwd("non-utf8");
        let error = run_process_with_limits(
            Path::new("/bin/sh"),
            &root,
            &fake_sh_argv("printf '\\377'"),
            Duration::from_secs(1),
            1024,
        )
        .unwrap_err();
        assert!(error.contains("不是有效 UTF-8"));
        let _ = fs::remove_dir_all(&root);
    }
}
