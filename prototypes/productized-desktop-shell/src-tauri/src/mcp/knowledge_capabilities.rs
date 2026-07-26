//! Fixed-vault, read-only knowledge capability handlers for the shared
//! supervisor conversation profile.
//!
//! This module deliberately owns neither a vault path nor a write path. The
//! only production root comes from `knowledge_vault`, and every request is
//! schema-validated again here before it can touch the filesystem.

use super::McpServerConfig;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const MAX_QUERY_BYTES: usize = 256;
const MAX_SEARCH_RESULTS: usize = 20;
const MAX_CITATIONS: usize = 16;
const VAULT_REFERENCE: &str = "syn-managed-markdown-vault";

pub(super) fn search_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"query": {"type": "string", "maxLength": MAX_QUERY_BYTES}},
        "required": ["query"],
        "additionalProperties": false
    })
}

pub(super) fn read_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"relative_path": {"type": "string"}},
        "required": ["relative_path"],
        "additionalProperties": false
    })
}

pub(super) fn open_input_schema() -> Value {
    read_input_schema()
}

pub(super) fn cite_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "relative_paths": {
                "type": "array",
                "minItems": 1,
                "maxItems": MAX_CITATIONS,
                "items": {"type": "string"}
            }
        },
        "required": ["relative_paths"],
        "additionalProperties": false
    })
}

pub(super) fn search(arguments: &Value) -> Result<Value, String> {
    let query = require_query(arguments)?;
    let vault_root = production_vault_root();
    search_at(&vault_root, &query)
}

pub(super) fn read(arguments: &Value) -> Result<Value, String> {
    let relative_path = require_markdown_relative_path(arguments)?;
    let vault_root = production_vault_root();
    read_at(&vault_root, &relative_path)
}

pub(super) fn open(config: &McpServerConfig, arguments: &Value) -> Result<Value, String> {
    let relative_path = require_markdown_relative_path(arguments)?;
    let vault_root = production_vault_root();
    open_at(config, &vault_root, &relative_path)
}

fn open_at(
    config: &McpServerConfig,
    vault_root: &Path,
    relative_path: &str,
) -> Result<Value, String> {
    // Do not claim a native view opened from this child process.  The fixed
    // host relay revalidates the same projection, waits for the UI's exact
    // acknowledgement, and returns only after the selected note has focus.
    read_document_projection_at(&vault_root, &relative_path)?;
    let opened = crate::knowledge_open_relay::dispatch_from_mcp(config, &relative_path)?;
    Ok(json!({
        "vault": VAULT_REFERENCE,
        "relative_path": opened.relative_path,
        "target": "syn_native_view",
        "dispatch_status": "opened",
        "opened": true,
        "intent_id": opened.intent_id,
        "external_open_requested": false,
        "knowledge_written": false
    }))
}

pub(super) fn cite(arguments: &Value) -> Result<Value, String> {
    let relative_paths = require_markdown_relative_paths(arguments)?;
    let vault_root = production_vault_root();
    cite_at(&vault_root, &relative_paths)
}

fn production_vault_root() -> PathBuf {
    crate::knowledge_vault::workspace_vault_root()
}

fn require_exact_object<'a>(
    arguments: &'a Value,
    required_keys: &[&str],
) -> Result<&'a Map<String, Value>, String> {
    let object = arguments
        .as_object()
        .ok_or_else(|| "知识工具参数必须是对象，已拒绝。".to_string())?;
    if object.len() != required_keys.len()
        || required_keys.iter().any(|key| !object.contains_key(*key))
        || object
            .keys()
            .any(|key| !required_keys.iter().any(|required| key == required))
    {
        return Err("知识工具参数必须恰好匹配固定 schema，已拒绝额外字段。".to_string());
    }
    Ok(object)
}

fn require_markdown_relative_path(arguments: &Value) -> Result<String, String> {
    let object = require_exact_object(arguments, &["relative_path"])?;
    let relative_path = object
        .get("relative_path")
        .and_then(Value::as_str)
        .ok_or_else(|| "知识工具缺少精确 relative_path，已拒绝。".to_string())?;
    validate_knowledge_markdown_relative_path(relative_path)
}

fn require_query(arguments: &Value) -> Result<String, String> {
    let object = require_exact_object(arguments, &["query"])?;
    let query = object
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| "知识搜索缺少精确 query，已拒绝。".to_string())?;
    if query.is_empty()
        || query.trim() != query
        || query.len() > MAX_QUERY_BYTES
        || query.chars().any(char::is_control)
        || query.contains("--")
        || query.contains(['*', '?', '[', ']', '\'', '"'])
    {
        return Err(
            "知识搜索 query 只能是受限的普通文本；通配符、控制参数和额外选项均被拒绝。".to_string(),
        );
    }
    Ok(query.to_string())
}

fn require_markdown_relative_paths(arguments: &Value) -> Result<Vec<String>, String> {
    let object = require_exact_object(arguments, &["relative_paths"])?;
    let values = object
        .get("relative_paths")
        .and_then(Value::as_array)
        .ok_or_else(|| "知识引用缺少 relative_paths 数组，已拒绝。".to_string())?;
    if values.is_empty() || values.len() > MAX_CITATIONS {
        return Err("知识引用 relative_paths 数量必须在安全上限内。".to_string());
    }
    let mut seen = BTreeSet::new();
    let mut relative_paths = Vec::with_capacity(values.len());
    for value in values {
        let relative_path = value
            .as_str()
            .ok_or_else(|| "知识引用 relative_paths 只能包含精确字符串。".to_string())?;
        let relative_path = validate_knowledge_markdown_relative_path(relative_path)?;
        if !seen.insert(relative_path.clone()) {
            return Err("知识引用 relative_paths 不能含空白变体或重复项。".to_string());
        }
        relative_paths.push(relative_path);
    }
    Ok(relative_paths)
}

pub(crate) fn validate_knowledge_markdown_relative_path(
    raw_relative_path: &str,
) -> Result<String, String> {
    if raw_relative_path.trim() != raw_relative_path {
        return Err("知识工具 relative_path 不能含首尾空白或路径变体，已拒绝。".to_string());
    }
    let relative_path = crate::knowledge_vault::validate_workspace_relative_path(raw_relative_path)
        .map_err(|_| {
            "知识工具 relative_path 必须是固定 Syn vault 内受限的精确 Markdown 相对路径。"
                .to_string()
        })?;
    let file_name = relative_path.file_name();
    if !file_name.ends_with(".md") || file_name.len() <= ".md".len() {
        return Err("知识工具 relative_path 只允许固定 Syn vault 内的 .md 文件。".to_string());
    }
    Ok(relative_path.as_str().to_string())
}

fn serialized_projection<T: serde::Serialize>(projection: &T) -> Result<Value, String> {
    serde_json::to_value(projection)
        .map_err(|_| "知识工作区只读投影无法序列化，已闭锁。".to_string())
}

fn read_document_projection_at(vault_root: &Path, relative_path: &str) -> Result<Value, String> {
    let document = crate::knowledge_index::workspace_read_markdown_at(vault_root, relative_path)?;
    serialized_projection(&document)
}

fn required_projection_string(document: &Value, field: &str) -> Result<String, String> {
    document
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "知识工作区只读投影缺少必填字段，已闭锁。".to_string())
}

fn required_projection_value(document: &Value, field: &str) -> Result<Value, String> {
    document
        .get(field)
        .cloned()
        .ok_or_else(|| "知识工作区只读投影缺少必填字段，已闭锁。".to_string())
}

fn read_at(vault_root: &Path, relative_path: &str) -> Result<Value, String> {
    let mut document = read_document_projection_at(vault_root, relative_path)?;
    let object = document
        .as_object_mut()
        .ok_or_else(|| "知识工作区只读投影格式异常，已闭锁。".to_string())?;
    object.insert(
        "vault".to_string(),
        Value::String(VAULT_REFERENCE.to_string()),
    );
    object.insert("knowledge_written".to_string(), Value::Bool(false));
    Ok(document)
}

fn search_at(vault_root: &Path, query: &str) -> Result<Value, String> {
    let response = crate::knowledge_index::workspace_search_at(vault_root, query)?;
    let projection = serialized_projection(&response)?;
    let results = projection
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| "知识工作区搜索投影格式异常，已闭锁。".to_string())?
        .iter()
        .take(MAX_SEARCH_RESULTS)
        .cloned()
        .collect::<Vec<_>>();
    Ok(json!({
        "vault": VAULT_REFERENCE,
        "results": results,
        "knowledge_written": false
    }))
}

fn cite_at(vault_root: &Path, relative_paths: &[String]) -> Result<Value, String> {
    let mut citations = Vec::with_capacity(relative_paths.len());
    for relative_path in relative_paths {
        let document = read_document_projection_at(vault_root, relative_path)?;
        citations.push(json!({
            "relative_path": required_projection_string(&document, "relative_path")?,
            "title": required_projection_string(&document, "title")?,
            "mtime_ms": required_projection_value(&document, "mtime_ms")?
        }));
    }
    Ok(json!({
        "vault": VAULT_REFERENCE,
        "citations": citations,
        "knowledge_written": false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    const MAX_TITLE_CHARS: usize = 256;

    struct VaultFixture {
        root: PathBuf,
    }

    impl VaultFixture {
        fn new(tag: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "syn-knowledge-capability-{tag}-{}-{}",
                std::process::id(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).expect("create temporary Syn vault");
            Self { root }
        }

        fn write_markdown(&self, relative_path: &str, body: &str) {
            let path = self.root.join(relative_path);
            fs::create_dir_all(path.parent().expect("nested test note parent"))
                .expect("create nested test note parent");
            fs::write(path, body).expect("write temporary Syn note");
        }
    }

    impl Drop for VaultFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn schemas_and_runtime_arguments_reject_variants_wildcards_and_extra_fields() {
        assert_eq!(search_input_schema()["additionalProperties"], false);
        assert_eq!(read_input_schema()["additionalProperties"], false);
        assert_eq!(open_input_schema()["additionalProperties"], false);
        assert_eq!(cite_input_schema()["additionalProperties"], false);
        assert_eq!(read_input_schema()["required"], json!(["relative_path"]));
        assert_eq!(cite_input_schema()["required"], json!(["relative_paths"]));

        for arguments in [
            json!({"query": "needle", "path": "/tmp/other"}),
            json!({"query": "needle*"}),
            json!({"query": "needle --limit=999"}),
            json!({"query": " needle"}),
            json!({"query": "needle\nother"}),
        ] {
            assert!(require_query(&arguments).is_err(), "{arguments}");
        }
        for arguments in [
            json!({"relative_path": "../outside.md"}),
            json!({"relative_path": "research/Note*.md"}),
            json!({"relative_path": "research/Note.md", "command": "open"}),
            json!({"relative_path": " research/Note.md"}),
            json!({"relative_path": "research/Note.canvas"}),
            json!({"slug": "legacy"}),
        ] {
            assert!(
                require_markdown_relative_path(&arguments).is_err(),
                "{arguments}"
            );
        }
        assert!(require_markdown_relative_paths(&json!({
            "relative_paths": ["research/One.md", "research/One.md"]
        }))
        .is_err());
    }

    #[test]
    fn nested_fixed_vault_read_search_and_cite_are_bounded_and_leave_notes_unchanged() {
        let vault = VaultFixture::new("read-search");
        vault.write_markdown(
            "research/Alpha.md",
            "---\ntags: [syn]\n---\n# Alpha title\n\nneedle in the first note\n",
        );
        vault.write_markdown("Beta.md", "# Beta title\n\nother text\n");
        let alpha_path = vault.root.join("research/Alpha.md");
        let before = fs::read(&alpha_path).expect("read before");

        let read = read_at(&vault.root, "research/Alpha.md").expect("read fixed-vault note");
        assert_eq!(read["vault"], VAULT_REFERENCE);
        assert_eq!(read["relative_path"], "research/Alpha.md");
        assert!(read["body"].as_str().unwrap().contains("needle"));

        let search = search_at(&vault.root, "needle").expect("search fixed-vault notes");
        let results = search["results"].as_array().expect("result array");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["relative_path"], "research/Alpha.md");

        let cite = cite_at(&vault.root, &["research/Alpha.md".to_string()])
            .expect("cite fixed-vault note");
        assert_eq!(cite["knowledge_written"], false);
        assert_eq!(cite["citations"][0]["relative_path"], "research/Alpha.md");
        assert_eq!(
            fs::read(alpha_path).expect("read after"),
            before,
            "all four knowledge actions remain vault-read-only"
        );
    }

    #[test]
    fn case_variant_symlink_and_oversize_notes_fail_closed() {
        let vault = VaultFixture::new("closed");
        vault.write_markdown("research/ExactCase.md", "# Exact\n\nbody\n");
        assert!(read_at(&vault.root, "research/exactcase.md").is_err());
        vault.write_markdown(
            "research/LongTitle.md",
            &format!("# {}\n\nneedle\n", "x".repeat(MAX_TITLE_CHARS + 10)),
        );
        let bounded_title =
            read_at(&vault.root, "research/LongTitle.md").expect("read bounded title");
        assert!(
            bounded_title["title"].as_str().unwrap().chars().count() <= MAX_TITLE_CHARS,
            "title output must remain bounded even when the Markdown heading is not"
        );
        vault.write_markdown(
            "research/Large.md",
            &"x".repeat(crate::knowledge_vault::MAX_MARKDOWN_BYTES as usize + 1),
        );
        assert!(read_at(&vault.root, "research/Large.md").is_err());

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                vault.root.join("research/ExactCase.md"),
                vault.root.join("research/Linked.md"),
            )
            .expect("create test-only symlink");
            assert!(read_at(&vault.root, "research/Linked.md").is_err());
            let search = search_at(&vault.root, "body").expect("skip unsafe symlink");
            assert!(search["results"].as_array().unwrap().iter().all(|result| {
                result["relative_path"] != Value::String("research/Linked.md".to_string())
            }));
        }
    }

    fn supervisor_config_without_relay() -> McpServerConfig {
        McpServerConfig {
            role: crate::mcp::McpRole::SupervisorOrchestrator,
            run_id: "supervisor-conversation:knowledge-open-fixture".to_string(),
            node_id: None,
            supervisor_workflow_state_path: None,
            supervisor_quota_limits: None,
            knowledge_open_relay: None,
        }
    }

    fn supervisor_config_with_unavailable_relay(vault: &VaultFixture) -> McpServerConfig {
        let endpoint = vault.root.join("knowledge-open-relay-missing.sock");
        McpServerConfig {
            role: crate::mcp::McpRole::SupervisorOrchestrator,
            run_id: "supervisor-conversation:knowledge-open-fixture".to_string(),
            node_id: None,
            supervisor_workflow_state_path: None,
            supervisor_quota_limits: None,
            knowledge_open_relay: Some(
                crate::knowledge_open_relay::KnowledgeOpenRelayMcpConfig::from_mcp_arguments(
                    endpoint.display().to_string(),
                    "a".repeat(64),
                    "turn:knowledge-open-fixture".to_string(),
                    "project:knowledge-open-fixture".to_string(),
                )
                .expect("test relay configuration is syntactically valid"),
            ),
        }
    }

    #[test]
    fn knowledge_open_requires_a_host_owned_relay_and_never_claims_opened_without_ack() {
        let vault = VaultFixture::new("open");
        vault.write_markdown("research/OpenMe.md", "# Open me\n");
        let path = vault.root.join("research/OpenMe.md");
        let before = fs::read(&path).expect("read before native intent");

        assert!(
            open_at(
                &supervisor_config_without_relay(),
                &vault.root,
                "research/OpenMe.md",
            )
            .is_err(),
            "without a host-issued relay config, knowledge_open must fail closed"
        );
        assert_eq!(fs::read(path).expect("read after rejected open"), before);
    }

    #[test]
    fn knowledge_open_with_no_listener_fails_closed_and_leaves_notes_unchanged() {
        let vault = VaultFixture::new("open-no-listener");
        vault.write_markdown("research/OpenMe.md", "# Open me\n");
        let path = vault.root.join("research/OpenMe.md");
        let before = fs::read(&path).expect("read before unavailable relay");

        assert!(
            open_at(
                &supervisor_config_with_unavailable_relay(&vault),
                &vault.root,
                "research/OpenMe.md",
            )
            .is_err(),
            "a syntactically valid relay config without a listener cannot claim opened"
        );
        assert_eq!(
            fs::read(path).expect("read after unavailable relay"),
            before,
            "failed knowledge_open stays read-only"
        );
    }
}
