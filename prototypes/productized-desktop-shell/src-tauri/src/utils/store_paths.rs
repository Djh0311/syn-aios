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
