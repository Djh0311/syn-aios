# RU1/RU2 阻断分类记录 v1

日期：2026-06-17

阶段：Real-Use De-risk（RU，真实使用去险）

状态：`blocked_classified`

## 拍板摘要

本记录不是完成报告，而是把 RU 执行线在安全边界内能核到的事实与阻断点固定下来：真实 `mariotest` 项目和真实 workbench state root 已只读核实；GUI 真机跑通与经 M2 门写第一条正式记忆未执行，因为当前默认 Tauri snapshot 路径会读 `/Users/yoyi/.codex`，命中 RU 硬封印。若不接受本阻断分类，继续硬跑会把“不得读 `.codex` / 不得造合成记忆”的红线变成纸面约束。

一句话判据：RU 当前能否继续，先问“是否有一条不读 `.codex`、不改源码、且经 M2 门写真实正式记忆的产品入口”；没有则停，不能用手改 JSON 或启动默认 GUI 冒充完成。

## 读取范围与边界

- 已读：`handoffs/2026-06-17-real-use-de-risk-ru-stage-claude-to-codex-kickoff-v1.md`、`docs/plans/2026-06-17-real-use-de-risk-dogfood-stage-plan-v1.md`、`CURRENT.md` 首条、`AUTHORITY.md` 当前阶段段落、`AGENTS.md`、`skills/using-superpowers/SKILL.md`、相关 Tauri / memory 入口源码。
- 已只读清点：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`、`/Users/yoyi/Documents/mario test`。
- 未读写：`/Users/yoyi/.codex`、secret/token/.env/keychain/OAuth/provider credential、full transcript、rollout、prompt body。
- 未执行：`tauri dev`、真实 `codex exec` / `codex exec resume`、K3-B1/K3-B2、真实 retry/stop/restart/resume、R3 产品全局 read/write 切换。
- 未写：真实 workbench state root、`mariotest` 项目文件、产品源代码、`CURRENT.md`。

## 当前仓库与工作树

- `git rev-parse --short HEAD`：`512c047`
- 开工时已有咨询线未提交改动，未覆盖：
  - `M AUTHORITY.md`
  - `M CURRENT.md`
  - `?? docs/plans/2026-06-17-real-use-de-risk-dogfood-stage-plan-v1.md`
  - `?? handoffs/2026-06-16-ui-xuanji-layout-relayout-handoff-to-new-conversation-v1.md`
  - `?? handoffs/2026-06-17-real-use-de-risk-ru-stage-claude-to-codex-kickoff-v1.md`

## Pre-work 检查

`node scripts/harness/capability-scan.js --target .`

```text
Harness capability scan: /Users/yoyi/workspace/product-line

PASS (7)
  - Harness config readable: harness.config.json
  - Project type inference: mixed (harness config project.type)
  - Test files detected: 5
  - Harness rule file found: AGENTS.md
  - Harness rule file found: codex-multi-agent-safe-collaboration.md
  - Harness rule file found: skills/using-superpowers/SKILL.md
  - Runtime docs present: 11/11

WARN (10)
  - No package.json found; command detection is limited to files and PATH
  - No package manager field or lockfile found
  - No lint script detected
  - No typecheck script detected
  - No test script detected
  - No e2e script detected in shallow scan
  - No build script detected
  - No dev script detected
  - No E2E/browser test assets detected in shallow scan
  - No CI workflow detected

FAIL (0)
  None
```

`node scripts/harness/guard-state-files.js --target .`

```text
Harness state-file guard: /Users/yoyi/workspace/product-line

PASS (19)
```

## RU1 只读事实：真实数据根与 mariotest 已存在

真实 workbench state root：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state
```

目录清单：

```text
total 4064
drwxr-xr-x@   5 yoyi  staff      160 Jun 10 01:24 .
drwxr-xr-x@   5 yoyi  staff      160 Jun 15 16:00 ..
drwxr-xr-x@ 227 yoyi  staff     7264 Jun 10 01:24 backups
-rw-r--r--@   1 yoyi  staff    15211 Jun 10 01:24 plan-authorizations.v1.json
-rw-r--r--@   1 yoyi  staff  2060497 May 31 19:30 workflow-state.v0.json
```

文件 hash：

```text
4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972  /Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e  /Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/plan-authorizations.v1.json
```

根目录一层文件：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/plan-authorizations.v1.json
```

记忆相关 sidecar 当前均不存在：

```text
memory-capture-events.v1.json: false
observations.v1.json: false
memory-candidates.v1.json: false
formal-memories.v1.json: false
memory-patterns.v1.json: false
memory-entity-relations.v1.json: false
memory-lint.v1.json: false
runtime-logs.v1.json: false
session-continuations.v1.json: false
blackboard-candidates.v1.json: false
project-consultation-proposals.v1.json: false
memory-capture-bus.v1.json: false
```

`mariotest` 项目目录：

```text
/Users/yoyi/Documents/mario test
```

目录清单：

```text
total 48
drwxr-xr-x   8 yoyi  staff   256 Jun  8 12:26 .
drwx------+ 11 yoyi  staff   352 May 31 18:26 ..
drwxr-xr-x  11 yoyi  staff   352 May 30 23:39 .git
drwxr-xr-x   5 yoyi  staff   160 Jun 10 06:12 .workbench
-rw-r--r--@  1 yoyi  staff   446 May 31 02:17 README.md
-rw-r--r--@  1 yoyi  staff  8932 May 31 02:21 game.js
-rw-r--r--@  1 yoyi  staff  1207 May 31 02:17 index.html
-rw-r--r--@  1 yoyi  staff  2120 May 31 02:17 styles.css
```

`.workbench` 已有历史探针文件：

```text
/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md
/Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md
/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md
```

这些是历史真实执行/探针产物；本 RU 窗口未复用它们作为“本窗已执行”证据。

## workflow-state 中的 mariotest 事实

解析 `workflow-state.v0.json` 得到：

```json
{
  "schema_version": "workflow_state_v0",
  "project": {
    "created_at": "1780156724812",
    "display_name": "mario test",
    "permission_level": "user_confirmed_write",
    "project_id": "project:users-yoyi-documents-mario-test",
    "root_path": "/Users/yoyi/Documents/mario test",
    "source_kind": "codex_index",
    "updated_at": "1780156724812",
    "warnings": [
      "project_directory_currently_only_git_metadata"
    ]
  },
  "workflow": {
    "created_at": "1780156724812",
    "entry_node_id": "workflow:users-yoyi-documents-mario-test:default:node:director",
    "model_policy": "codex_threads_user_confirmed",
    "permission_level": "user_confirmed_write",
    "project_id": "project:users-yoyi-documents-mario-test",
    "source_kind": "workspace_state",
    "state": "draft",
    "title": "mario test 四角色编排测试工作流",
    "updated_at": "1780156724812",
    "warnings": [
      "real_codex_resume_requires_separate_user_approval"
    ],
    "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
    "workflow_version": 1
  }
}
```

Workflow topology：

```json
{
  "node_count": 7,
  "edge_count": 7,
  "node_ids": [
    "workflow:users-yoyi-documents-mario-test:default:node:director",
    "workflow:users-yoyi-documents-mario-test:default:node:codex-dev",
    "workflow:users-yoyi-documents-mario-test:default:node:validation",
    "workflow:users-yoyi-documents-mario-test:default:node:review",
    "workflow:users-yoyi-documents-mario-test:default:node:task",
    "workflow:users-yoyi-documents-mario-test:default:node:handoff",
    "workflow:users-yoyi-documents-mario-test:default:node:evidence"
  ],
  "edge_count_confirmed": 7
}
```

Audit：

```text
audit_events_total: 356
mariotest_audit_events_by_project_or_workflow_id: 253
```

## 阻断证据 A：默认 GUI / snapshot 会读 .codex

关键源码路径：

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs:4`：`load_workbench_snapshot` 调 `build_snapshot(&state, &index, &tasks_text)`。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs:12`：`query_workbench_page_read_model` 也先构造 snapshot。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:1308`：`build_snapshot()` 固定传 `SessionSourceMode::RealWithSqliteFallback`。
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs:14`：`RealWithSqliteFallback` 进入 `load_sessions_from_sqlite_or_index`。
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs:22`：该函数调用 `codex_db::default_state_db_path()`。
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs:1`：文件注释明确是 direct read from Codex sqlite。
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs:32`：`default_state_db_path()` 从 `$HOME/.codex` 找 `state_*.sqlite`，找不到则回落 `$HOME/.codex/state_5.sqlite`。
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs:60`：`read_threads()` 以 read-only sqlite 打开该路径。

因此：启动默认 GUI 或调用依赖 snapshot 的默认页面读模型，会触发对 `/Users/yoyi/.codex` 的路径访问。RU kickoff 与计划正本硬封印写明“不读写 `/Users/yoyi/.codex`”，所以本窗口不能用默认 GUI/tauri dev 来声称 RU1 真机跑通。

## 阻断证据 B：RU2 缺少不经 GUI 的安全 M2 产品入口

现有 M2 采纳链路存在，但暴露为 Tauri command / 前端 pending action：

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs:553`：`capture_memory_event` 写 capture / observation / candidate。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs:754`：`adopt_memory_candidate_to_formal_memory` 经 `adopt_memory_candidate_to_formal_memory_at`。
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs:13`：`adopt_memory_candidate_to_formal_memory_at` 先跑 memory lint guard，再调用 `memory_candidate_store::adopt_candidate_to_formal_memory`。
- `prototypes/productized-desktop-shell/src/App.tsx:339` / `348`：正式记忆采纳通过 pending action 调 `adoptMemoryCandidateToFormalMemory`。
- `prototypes/productized-desktop-shell/src/App.tsx:504`：L5 日常 capture 入口在 operation-control decision 后调用 `captureMemoryEvent`。

但本次没有发现已存在的 RU 专用 CLI / ignored runner / MCP 命令，能在不启动默认 GUI、不读 `.codex`、不改源码的情况下，按 M2 用户确认门写入真实 FormalMemory。直接手工写 `formal-memories.v1.json` 或 `memory-candidates.v1.json` 会绕过 M2 门，不符合 RU2 红线。

## RU 分项结论

- RU1：`partial_readonly_verified`。真实 state root、真实 `mariotest` 项目、draft workflow、节点/边、历史 audit 已只读核实；但“GUI 真机跑通 + 重开仍在”未执行，因为默认 GUI snapshot 路径会读 `.codex`。
- RU2：`blocked_not_executed`。当前正式记忆 sidecar 不存在；未写 capture / observation / candidate / FormalMemory；缺少不读 `.codex` 且经 M2 门的安全非 GUI 入口。
- RU3：`blocked_deferred`。只能产出阻断分类与下一步建议；不能判定 L5 完工线达成，也不能给“开 B”正向建议。

## 建议的下一步

二选一，需咨询线/用户另拍：

1. 新开窄代码任务：提供 RU/Dogfood 后端入口或测试 runner，要求显式传入 confirmed real workflow-state path，禁止读取 `.codex`，只允许 `capture -> observation -> candidate -> M2 adoption`，并带负向测试与真实窗口前置确认。
2. 用户手动 GUI 实操：只有在用户明确接受“默认 GUI 会 read-only 访问 `/Users/yoyi/.codex/state_*.sqlite`”这一事实后，才可由用户在场手动跑；这会改变 RU 当前硬封印，不能由执行线自行决定。

## 不可声称

- 不可声称 RU1 真机跑通完成。
- 不可声称已写入第一条真实正式记忆。
- 不可声称 L5 完工线已兑现。
- 不可声称 B 可开或 K3-B1/K3-B2 可继续。
- 不可声称产品全局读写已切 DB、真库 C 已执行、真实 retry/stop/restart/resume 已执行。
- 不可声称本窗口触碰或读取了 `/Users/yoyi/.codex`。
