// M5-B Batch 1 DB-primary bridge. Callers remain explicit so the storage-mode flag cannot
// silently widen into Batch 2 paths. JSON-only and Blocked modes retain the established writer.

fn write_m5b_batch1_workflow_state(
    path: &Path,
    phase: &str,
    value: &Value,
) -> Result<(), String> {
    if let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(path)?
    {
        return write_m5b_batch1_workflow_state_db_primary(path, phase, value, &repository);
    }

    write_validated_workflow_state(path, value)
}

fn write_m5b_batch1_workflow_state_db_primary(
    path: &Path,
    phase: &str,
    value: &Value,
    repository: &crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
) -> Result<(), String> {
    let before = read_workflow_state_value(path)?;
    repository.record_workflow_state_delta_with_audit(&before, value, None)?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(path, phase, || {
        write_validated_workflow_state(path, value)
    })
}

// Batch 2 remains an explicit opt-in surface. Keep this separate from Batch 1 so later changes
// cannot silently widen the DB-primary flag to low-frequency write paths.
fn write_m5b_batch2_workflow_state(
    path: &Path,
    phase: &str,
    value: &Value,
) -> Result<(), String> {
    if let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(path)?
    {
        return write_m5b_batch2_workflow_state_db_primary(path, phase, value, &repository);
    }

    write_validated_workflow_state(path, value)
}

fn write_m5b_batch2_workflow_state_db_primary(
    path: &Path,
    phase: &str,
    value: &Value,
    repository: &crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
) -> Result<(), String> {
    let before = read_workflow_state_value(path)?;
    repository.record_workflow_state_delta_with_audit(&before, value, None)?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(path, phase, || {
        write_validated_workflow_state(path, value)
    })
}
