// A·存储模式的**只读**访问器（`system_status_read_model` 用）。
//
// 任务包：tasks/2026-07-15-backend-ui-support-readmodels-package-v1.md §A
//
// 为什么单独成文件：父文件 `workbench_sqlite_storage_mode.rs` 已贴着 shape gate 的 3000 行上限
// （直接加这段就破线·gate 当场抓到），故照 `m5c` 子模块先例拆出来。**只读·零语义改动**：
// 不建缓存条目、不改健康态、不触发对账、不碰模式缓存——读模型不得有副作用。

use super::{health_cache, DbPrimaryHealth};
use std::path::Path;

/// DB 主写健康的只读快照：照原样报 `health_cache()` 的现有条目。
///
/// - `Some(Ok(()))`  = 启动对账绿（Ready）
/// - `Some(Err(原因))` = 已冻结降级（Blocked·原因是后端写好的人话串）
/// - `None`          = **本进程尚未跑启动对账**（`initialize_for_startup` 没走过）。
///   注意 None **不是**「健康」：与 `primary_repository_for_write` 对 None 的保守口径（拒写）
///   同向，调用方一律按「判不了 → 保守」处理，别当绿灯。
pub(crate) fn db_primary_health_snapshot(workflow_state_path: &Path) -> Option<Result<(), String>> {
    health_cache()
        .lock()
        .expect("storage mode health lock")
        .get(workflow_state_path)
        .map(|health| match health {
            DbPrimaryHealth::Ready => Ok(()),
            DbPrimaryHealth::Blocked(reason) => Err(reason.clone()),
        })
}
