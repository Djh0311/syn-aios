# Final Skeleton 01 Audit Helper Slice v1 Evidence

日期：2026-06-01

## 本轮结论

先说薄弱点：本轮只迁移了一类低风险审计构造，不是事件账本完整迁移。

已完成：

- 将 `workflow_permission_decision_recorded` 审计事件构造迁移到 `workflow_audit.rs`。
- `record_workflow_permission_decision_at` 的真实写入路径已改为调用 `workflow_audit::workflow_permission_decision_recorded`。
- 新增字段一致性测试，确认 event_id、event_type、target_ref、actor_ref、source_kind、permission_level、before_state、after_state、created_at、reason 没丢。
- 未改 workflow state JSON 结构。
- 未改状态机。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。

## 依据

| 文件 | 用途 |
|---|---|
| `tasks/2026-06-01-final-skeleton-01-audit-helper-slice-v1.md` | 本小切片执行入口。 |
| `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md` | 总执行包和切片顺序。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 原权限确认 audit 调用点和测试位置。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs` | 新增审计 helper。 |
| `decisions/2026-06-01-architecture-module-split-guardrail-v1.md` | 不改状态机、不改 JSON、不碰真实 Codex 的边界。 |

## 原审计字段

`workflow_permission_decision_recorded` 原字段：

| 字段 | 原值规则 | 本轮状态 |
|---|---|---|
| `event_id` | `audit:workflow-permission-decision:<stable request id>:<timestamp>` | 保持。 |
| `event_type` | `workflow_permission_decision_recorded` | 保持。 |
| `target_ref` | `request.request_id` | 保持。 |
| `actor_ref` | `user_confirmed_desktop_shell` | 保持。 |
| `source_kind` | `workspace_state_permission_queue` | 保持。 |
| `permission_level` | `user_confirmed_write` | 保持。 |
| `before_state` | 权限请求原状态，默认 `pending` | 保持。 |
| `after_state` | `approved` 或 `rejected` | 保持。 |
| `created_at` | 当前 timestamp | 保持。 |
| `reason` | `用户确认记录权限请求结论；不启动 Codex、不 resume、不发送消息。` | 保持。 |

## 改动

| 文件 | 改动 |
|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs` | 新增 `WorkflowPermissionDecisionRecordedAudit` 和 `workflow_permission_decision_recorded`。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | `record_workflow_permission_decision_at` 改为调用审计 helper；新增 `workflow_audit_helper_preserves_permission_decision_fields` 测试。 |

## 测试结果

聚焦验证：

| 命令 | 结果 |
|---|---|
| `rustfmt --check src/workflow_audit.rs` | 通过。 |
| `cargo test --lib workflow_audit_helper_preserves_permission_decision_fields` | 通过。 |
| `cargo test --lib workflow_permission_decision_records_audit_through_control_core` | 通过。 |

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
| 不改审计字段含义 | 已遵守。 |
| 不改 workflow state JSON 结构 | 已遵守。 |
| 不改状态机 | 已遵守。 |
| 不执行真实 `codex exec` / `codex exec resume` | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不做黑板写入 | 已遵守。 |
| 不写正式记忆 | 已遵守。 |
| 不迁移数据库 | 已遵守。 |

## 不接受为

不接受为：

- 事件账本完整迁移。
- 所有审计构造完成拆分。
- 控制核心最终版。
- 黑板持久写入完成。
- 真实业务自动编排完成。

## 下一步

继续执行普通小任务：

- `final-skeleton-02-read-model-derivation-slice-v1`
