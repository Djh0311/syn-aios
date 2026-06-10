# Final Skeleton 02 Read Model Derivation Slice v1 Evidence

日期：2026-06-01

## 本轮结论

先说薄弱点：本轮只迁移了一个读模型派生函数，读模型体系还没有完整拆出。

已完成：

- 将 `derive_workflow_ledger_entries` 的实际派生逻辑迁移到 `workflow_read_model.rs`。
- `lib.rs` 保留同名 wrapper，调用 `workflow_read_model::derive_workflow_ledger_entries`。
- 为避免大范围迁移私有 helper，本轮通过 `WorkflowLedgerDerivationFns` 注入 `optional_string_from`、`string_array`、`i64_value`、`ledger_entry_type_from_audit`、`compact_ledger_summary`。
- 保持账本派生结果一致。
- 未改 UI 展示语义。
- 未改底层事实。
- 未写 workflow state。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。

## 依据

| 文件 | 用途 |
|---|---|
| `tasks/2026-06-01-final-skeleton-02-read-model-derivation-slice-v1.md` | 本小切片执行入口。 |
| `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md` | 总执行包和切片顺序。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 原 `derive_workflow_ledger_entries` 位置和 wrapper。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs` | 新读模型派生实现。 |
| `evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` | 上一轮已建立 `workflow_read_model.rs` 的依据。 |

## 迁移范围

| 项 | 本轮处理 |
|---|---|
| audit events -> ledger entries | 已迁移到 `workflow_read_model.rs`。 |
| node dispatches -> ledger entries | 已迁移到 `workflow_read_model.rs`。 |
| director reviews -> ledger entries | 已迁移到 `workflow_read_model.rs`。 |
| permission requests -> ledger entries | 已迁移到 `workflow_read_model.rs`。 |
| `derive_workflow_read_model` | 未迁移，仍在 `lib.rs`。 |
| 异常、子汇报、状态机、验收场景派生 | 未迁移，避免扩大范围。 |

## 改动

| 文件 | 改动 |
|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs` | 新增 `WorkflowLedgerDerivationFns` 和 `derive_workflow_ledger_entries`。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | `derive_workflow_ledger_entries` 改为 wrapper，调用读模型模块。 |

## 测试结果

聚焦验证：

| 命令 | 结果 |
|---|---|
| `rustfmt --check src/workflow_read_model.rs` | 先失败后只格式化该新增/修改模块，再次通过。 |
| `cargo test --lib workflow_ledger_derives_summary_entries_without_tool_output_fulltext` | 通过。 |

完整验证：

| 命令 | 结果 |
|---|---|
| `npm run typecheck` | 通过。 |
| `npm run test:offline-interaction` | 通过，`offline interaction tests passed: 2`。 |
| `npm run build` | 通过；仍有既有 Vite chunk 大小 warning。 |
| `cargo test --lib` | 通过，88 passed、0 failed、1 ignored；仍有既有 warning：`JsonRpcError::invalid_params` 未使用。 |

## 禁止事项执行情况

| 禁止项 | 结果 |
|---|---|
| 不改 UI 展示语义 | 已遵守。 |
| 不改底层事实 | 已遵守。 |
| 不写 workflow state | 已遵守。 |
| 不把缺字段补编成事实 | 已遵守。 |
| 不执行真实 `codex exec` / `codex exec resume` | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不迁移数据库 | 已遵守。 |

## 不接受为

不接受为：

- 读模型体系完整迁移。
- WorkbenchSnapshot 组装迁移完成。
- 事件账本最终版。
- 前端项目页收敛完成。
- 真实业务自动编排完成。

## 下一步

继续执行普通小任务：

- `final-skeleton-03-tauri-verification-line-design-v1`
