# R3 Level B B0 Preflight Report

日期：2026-06-14

状态：completed

窗口：B0 Preflight，仅只读侦察。未执行 B1+，未创建 DB，未切读，未停写 JSON / sidecar，未执行真实 Codex，未读取或写入 `/Users/yoyi/.codex`。

## 授权引用

`user-thread-2026-06-14-r3-level-b-b0-readonly-authorization`

用户已授权本窗口只读读取工作台自有状态文件以清点 / 计算 hash，并免去每步回问；硬底线仍为不碰 `/Users/yoyi/.codex`、secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout、prompt body。

## Git 和预检

`git log --oneline -6`：

```text
70f5557 docs: checkpoint r-u night target closure
6364519 docs: draft r-u-gate dedup guard options
599a35d docs: add r-u-gate dedup guard draft package
2813058 docs: checkpoint r-u4 normalization util dedup
16e96bd refactor: deduplicate rust normalization helpers
b5964b6 docs: add r-u4 normalization util dedup package
```

`git status --short`：

```text
?? docs/harness-script-audit-2026-06-14.md
?? docs/harness-source-package-audit-2026-06-14.md
?? docs/plans/2026-06-14-work-schedule-v1.md
```

说明：上述 3 个未跟踪文档为 B0 开始前已存在的外部变更，本窗口未读取、未修改、未提交。

`node scripts/harness/workbench-shape-gate.js --mode check`：

```text
Status: pass
Errors: 0
Warnings: 0
Git HEAD: 70f5557a70128ffa3d9cfced96b0690d62eb1628
Tauri commands: 97 total; 0 in lib.rs
Sidecar JSON kinds: 14 detected; 0 unknown
```

`git diff --check`：通过，输出为空。

## 路径来源和分类

Tauri 配置读取结果：

- `productName`: `CodexGovernanceWorkbench`
- `identifier`: `local.codex.governance.workbench`
- 窗口标题：`Codex 治理工作台`

代码默认状态路径读取结果：

- 前端默认路径：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- Tauri 默认路径：`$HOME/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

候选路径检查：

- `WORKBENCH_STATE_ROOT`: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`
- `WORKBENCH_STATE_ROOT` path sha256: `e5eda291644a3fac663598e2490020a03d5257da5549f5b278fc8db95d62f487`
- `Application Support/local.codex.governance.workbench`: 不存在。
- 邻接 app 数据目录：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/canvas-v1` 存在，但本窗口未将其纳入 workflow-state allowed 文件清点。

Denied paths 固定分类：

- `/Users/yoyi/.codex`
- secret / token / `.env`
- keychain / OAuth / provider credential
- full transcript / rollout body / prompt body
- 未列入 execution record 的任意用户项目源码目录
- 任意未列入 allowed roots 的路径

分类结论：`WORKBENCH_STATE_ROOT` 不位于 `/Users/yoyi/.codex`，不在固定禁读路径内；本窗口只读取该 root 下的 allowed JSON / sidecar 文件。

## Allowed 文件清单

source root aggregate hash before:

`2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53`

算法：对已存在 allowed 文件按 relative path 排序，拼接 `<path_sha256> <file_sha256> <relative_path>`，以 LF 连接后计算 SHA-256。

| 文件 | 状态 | size | sha256 | schema_version | revision |
| --- | --- | ---: | --- | --- | --- |
| `workflow-state.v0.json` | exists | 2060497 | `4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972` | `workflow_state_v0` | `1` |
| `plan-authorizations.v1.json` | exists | 15211 | `6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e` | `plan_authorization_store.v1` | `14` |
| `blackboard-candidates.v1.json` | missing | - | - | - | - |
| `formal-memories.v1.json` | missing | - | - | - | - |
| `memory-candidates.v1.json` | missing | - | - | - | - |
| `memory-capture-events.v1.json` | missing | - | - | - | - |
| `memory-entity-relations.v1.json` | missing | - | - | - | - |
| `memory-lint.v1.json` | missing | - | - | - | - |
| `memory-patterns.v1.json` | missing | - | - | - | - |
| `observations.v1.json` | missing | - | - | - | - |
| `project-proposals.v1.json` | missing | - | - | - | - |
| `real-execution-product-commands.v1.json` | missing | - | - | - | - |
| `runtime-log.v1.json` | missing | - | - | - | - |
| `runtime-logs.v1.json` | missing | - | - | - | - |
| `session-continuations.v1.json` | missing | - | - | - | - |

主状态文件顶层计数：

- projects: 5
- workflows: 5
- nodes: 35
- edges: 32
- audit_events: 356
- execution_attempts: 62
- workflow_execution_controls: 62
- workflow_machine_runs: 10
- workflow_node_dispatches: 118
- workflow_node_session_bindings: 36
- reviews: 11
- work_items: 12

`plan-authorizations.v1.json` 顶层计数：

- authorizations: 0
- audit_events: 14
- warnings: 1

## Production DB / Manifest Before

- 应用数据目录内 `*.sqlite`：未发现。
- `workbench-state.v1.sqlite`：未发现。
- production DB before: `missing_before=true`
- `WORKBENCH_STORAGE_ROOT`: B0 未在配置中发现可冻结的生产 SQLite storage root。
- backup manifest: `pending_before=true`
- rollback manifest: `pending_before=true`

## 真实状态判断

这不是空状态，也不是只有 fixture：`workflow-state.v0.json` 约 2.06 MB，已有 5 个项目、5 条 workflow、35 个节点、356 条审计事件和多类 workflow / dispatch 记录，属于已有真实工作台积累。

但这也不是完整 sidecar 积累：14 个 allowed sidecar 中只有 `plan-authorizations.v1.json` 存在，其余 13 个缺失；记忆、runtime log、session continuation、product command 等 sidecar 尚无真实文件积累。

B1 是否值得现在做：如果目标是验证“真实主状态迁移到 production DB”的路径，B1 有价值，因为主状态已有足够体量；如果目标是迁移完整中间版本全域 sidecar 生态，当前仍偏早，建议 B1 只作为受控 production apply 首次窗口，不应把它包装成完整存储切换或完整真实数据迁移。

## 本窗口未做

- 未创建 production DB。
- 未切 read path。
- 未停写 JSON / sidecar。
- 未写工作台状态文件。
- 未读取或写入 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未解锁多 agent 并行真实执行。
