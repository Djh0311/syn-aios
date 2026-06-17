# RU2/RU3 Confirmed-Path Memory Adoption Review - Aquinas v1

复核线：Aquinas
Agent id：019ece6b-4b39-7830-9553-86b979ec322c
日期：2026-06-17

STATUS: CLEAR

## Findings

- P0: 无
- P1: 无
- P2: 无
- P3: 无

## Scope / Boundary

已复核：

- `tasks/2026-06-17-real-use-de-risk-ru2-confirmed-path-memory-adoption-and-ru3-conclusion-v1.md`
- `evidence/2026-06-17-real-use-de-risk-ru2-ru3-confirmed-path-memory-adoption-v1.md`
- `handoffs/2026-06-17-real-use-de-risk-ru-stage-result-v1.md`
- `prototypes/productized-desktop-shell/src-tauri/src/ru_dogfood.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- 真实 state root 只读核验：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`

边界：

- 未读取或写入 `/Users/yoyi/.codex`。
- 未读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- 未启动 GUI / Tauri dev。
- 未执行 `codex exec` / `codex exec resume`。
- 未运行 runner/test。
- 未改产品代码，未 `git add` / `git commit`。
- 仅写入本 review 文件。

## Key Verification

- 新入口为 test-only confirmed-path：`ru_dogfood.rs` 仅由 `memory_context_entrypoints.rs` 通过 `#[cfg(test)] #[path = "ru_dogfood.rs"] mod ru_dogfood;` 注册；未发现新增 Tauri command、前端 UI 分支或产品运行时调用。`memory_context_entrypoints.rs` 只新增 test-only module 注册，并把既有 `adopt_memory_candidate_to_formal_memory_at` 提升为 `pub(crate)`，未改 helper 行为。
- `ru_dogfood.rs` 的 guard 与边界成立：入口要求 `CONFIRMED_USER_PRESENT_2026_06_17`，要求 `workflow_state_path == confirmed_workflow_state_path`，要求绝对路径、存在、文件名为 `workflow-state.v0.json`，并校验 project/workflow id 与 workflow-state 注册内容。denied path fragment 覆盖 `.codex`、secret/secrets、token/tokens、`.env`、keychain、oauth、credential/credentials、transcript/transcripts、prompt/prompts。未发现 `Command::new`、`codex exec`、`codex exec resume`、真实 stop/restart/kill 或 `.codex` 读取路径。
- FormalMemory 写入链路不是手写 JSON：源码顺序为 `memory_capture_bus::capture_event` -> `memory_candidate_store::record_decision(... CandidateConfirmed ...)` -> `adopt_memory_candidate_to_formal_memory_at`；M2 helper 内先跑 `run_memory_lint_at(... CandidateAdoptionGuard ...)`，再通过 `memory_candidate_store::adopt_candidate_to_formal_memory` 写 FormalMemory，并回写 candidate adoption link。
- 真实 state root 主文件 hash 与 evidence 一致且未变：
  - `workflow-state.v0.json` = `4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972`
  - `plan-authorizations.v1.json` = `6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e`
- 真实 state root 当前一层文件只包含两个主文件与 5 个允许记忆 sidecar：`memory-capture-events.v1.json`、`observations.v1.json`、`memory-candidates.v1.json`、`formal-memories.v1.json`、`memory-lint.v1.json`。sidecar hash 与 evidence 一致。
- 真实 FormalMemory 对齐：`formal-memories.v1.json` 有 1 条记录，`memory_id=mem:v1:1781630651485:8a3140d6102a2c7d`，`status=memory_active`，claim 为 `mario test RU 发现默认 GUI 真机路径会读 Codex state，记忆闭环需要 confirmed-path 入口。`，source_refs 指向 RU blocked evidence 与 observation，audit event 为 `memory_candidate_adopted_to_formal_memory`。
- candidate adoption link 对齐：`memory-candidates.v1.json` 中 `candidate_key=memcand:v1:d52ec5fb5378ffb013219d0d6bd1e6f4b9682c195c436fe6ac0640dd88dae55d`，状态为 `candidate_confirmed`；audit_refs 包含 `candidate_needs_review` 创建、`user:ru-dogfood` 确认、`project-director:ru-dogfood` 采纳三段；`adoption.adopted_memory_id` 指向目标 FormalMemory。
- lint 对齐：`memory-lint.v1.json` 中 1 次 run，`lint_intent=candidate_adoption_guard`，`actor_role=project_director`，`status=succeeded`，`blocking_count=0`，`finding_ids=[]`。
- capture / observation 对齐：`memory-capture-events.v1.json` 中 1 event，`candidate_policy=candidate_allowed`，指向 `observation_id=obs:v1:1781630651485:3313fafbc3954aa5` 与同一 candidate；`observations.v1.json` 中 observation 状态为 `candidate_created`，risk/sensitive 为 `low` / `internal`。
- RU3 结论诚实：evidence/handoff 只声称 “L5 真记忆完工线窄口径达成”，同时明确不接受为 GUI 真机体感已验、B 已解锁、真实 Codex 已执行、产品全局 read/write path 已切 DB、完整迁移或 JSON stop-write。
- 验证证据足够：evidence 记录了 `cargo test --lib ru_dogfood`（3 passed / 1 ignored）、真实 ignored runner（1 passed）、`memory_capture_bus`（8 passed）、`memory_daily_loop`（2 passed）、`memory_candidate`（9 passed）、`cargo test --lib`（521 passed / 22 ignored）、`cargo fmt -- --check`、`rustfmt ru_dogfood.rs`、shape gate pass（0 errors / 1 known Tauri command total warning）与 `git diff --check`。

## Read-Only Checks Run

- `sed -n` 读取任务包、evidence、handoff、`ru_dogfood.rs`、`memory_context_entrypoints.rs`。
- `rg -n` 检查 `ru_dogfood` 注册点、Tauri command / UI / exec / sensitive path 命中。
- `find` 只读列出真实 state root 一层文件。
- `shasum -a 256` 只读核验主文件与 5 个 sidecar hash。
- `jq` 只读抽取 capture、observation、candidate、formal memory、memory lint 关键字段。
- `git status --short`、`git diff --stat`、`git diff -- memory_context_entrypoints.rs` 只读核工作树范围。

## Conclusion

本包可接受为 RU2/RU3 confirmed-path dogfood 收口：真实 FormalMemory 通过 confirmed-path runner 经 capture -> observation -> candidate -> user confirm -> memory lint -> M2 adoption 写入，未发现手写 FormalMemory JSON、`.codex` 接触、真实 Codex 执行、GUI 启动或产品运行时入口扩张。提交前无必须修复项。
