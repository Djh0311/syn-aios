# Evidence: uiwork 旧 running run 清账 v1

## 薄弱点

- 这只是清理 workflow state 旧账，不是新的 UI 工作流执行。
- 没有删除历史 run，只把无步骤、无结束时间的旧 `running` run 标记为 `cancelled`。
- 使用 `cancelled` 而不是 `superseded`，依据是当前工作台已有 `cancelled` 状态标签，`superseded` 不是现成 UI 状态。

## 用户指令

用户要求：“清理掉”。

我的理解：允许写真实 workflow state，把 uiwork 旧 `running` run 收口；不执行任何 Codex 派发，不写 `/Users/yoyi/.codex`。

## 清理对象

- run id：`workflow-machine-run:workflow-users-yoyi-documents-uiwork-default:workflow-users-yoyi-documents-uiwork-default-inkwash-ui-replacement-v1:1780224885195`
- 清理前状态：`running`
- 清理前 `ended_at`：`null`
- 清理前 `rounds_completed`：`0`
- 清理前 `steps_count`：`0`
- 后续 accepted run：`workflow-machine-run:workflow-users-yoyi-documents-uiwork-default:workflow-users-yoyi-documents-uiwork-default-inkwash-ui-replacement-v1:1780225012481`

## 写入结果

- 写真实 workflow state：是。
- 写 `/Users/yoyi/.codex`：否。
- 执行 `codex exec` / `codex exec resume`：否。
- 读取敏感文件或完整 transcript：否。
- 删除 run 记录：否。

字段变化：

- `workflow_machine_runs[].state`: `running -> cancelled`
- `workflow_machine_runs[].ended_at`: `1780227015824`
- `workflow_machine_runs[].cleaned_up_at`: `1780227015824`
- `workflow_machine_runs[].cleanup_reason`: `stale_running_run_without_steps`
- `workflow_machine_runs[].superseded_by_run_id`: accepted run id
- `workflow_machine_runs[].warnings[]`: 增加清理说明
- `audit_events[]`: 增加 `workflow_machine_stale_run_cleaned`
- 顶层 `updated_at`: `1780227015824`

## 备份

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780227015824.json`

## 审计事件

- `audit:workflow-machine-stale-run-cleaned:1780227015824`

## 只读复核

uiwork 三条相关 run 当前状态：

- `1780224330335`: `needs_changes`，`steps_count=5`
- `1780224885195`: `cancelled`，`steps_count=0`，`superseded_by_run_id=1780225012481`
- `1780225012481`: `accepted`，`steps_count=5`

结论：旧 `running` run 已清理；最终 accepted run 未被改动。
