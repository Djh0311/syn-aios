use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub(crate) fn sidecar_path(
    workflow_state_path: &Path,
    sidecar_name: &str,
    store_name: &str,
) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导 {store_name} sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(sidecar_name))
}

pub(crate) fn runtime_artifact_dir(
    workflow_state_path: &Path,
    category: &str,
    run_id: &str,
) -> Result<PathBuf, String> {
    let state_parent = workflow_state_path.parent().ok_or_else(|| {
        format!(
            "workflow state 路径没有父目录，无法推导 {category} 运行材料目录：{}",
            workflow_state_path.display()
        )
    })?;
    let app_root =
        if state_parent.file_name().and_then(|name| name.to_str()) == Some("workflow-state") {
            state_parent.parent().unwrap_or(state_parent)
        } else {
            state_parent
        };
    Ok(app_root
        .join("runtime-artifacts")
        .join(category)
        .join(crate::utils::hash::short_hash(run_id)))
}

pub(crate) fn ensure_runtime_artifact_dir(
    workflow_state_path: &Path,
    category: &str,
    run_id: &str,
) -> Result<PathBuf, String> {
    let path = runtime_artifact_dir(workflow_state_path, category, run_id)?;
    fs::create_dir_all(&path).map_err(|error| {
        format!(
            "创建 {category} 运行材料目录失败 {}：{error}",
            path.display()
        )
    })?;
    #[cfg(unix)]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).map_err(|error| {
        format!(
            "收紧 {category} 运行材料目录权限失败 {}：{error}",
            path.display()
        )
    })?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_artifacts_are_outside_workflow_state_root_and_portable() {
        let path = Path::new("/tmp/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json");
        let output = runtime_artifact_dir(path, "supervisor", "supervisor:workflow:1")
            .expect("runtime artifact path");

        assert!(output.starts_with("/tmp/CodexGovernanceWorkbench/runtime-artifacts/supervisor"));
        assert!(!output.starts_with("/tmp/CodexGovernanceWorkbench/workflow-state"));
        assert!(!output.to_string_lossy().contains(':'));
    }

    #[cfg(unix)]
    #[test]
    fn runtime_artifact_directory_is_private() {
        let root = std::env::temp_dir().join(format!(
            "runtime-artifact-dir-{}",
            crate::unix_timestamp_nanos()
        ));
        let state_path = root.join("workflow-state/workflow-state.v0.json");
        let output = ensure_runtime_artifact_dir(&state_path, "supervisor", "run:private")
            .expect("private runtime artifact dir");
        let mode = fs::metadata(&output)
            .expect("runtime artifact metadata")
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(mode, 0o700);
        let _ = fs::remove_dir_all(root);
    }
}
