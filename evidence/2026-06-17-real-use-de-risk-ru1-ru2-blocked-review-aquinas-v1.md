# RU1/RU2 Blocked Classification Review - Aquinas v1

日期：2026-06-17
复核线：Aquinas
Agent id：019ece6b-4b39-7830-9553-86b979ec322c

STATUS: CLEAR

## Scope

已复核：

- `evidence/2026-06-17-real-use-de-risk-ru1-ru2-blocked-v1.md`
- `handoffs/2026-06-17-real-use-de-risk-ru-stage-blocked-result-v1.md`
- `handoffs/2026-06-17-real-use-de-risk-ru-stage-claude-to-codex-kickoff-v1.md`
- `docs/plans/2026-06-17-real-use-de-risk-dogfood-stage-plan-v1.md`
- `CURRENT.md` 首条
- 必要源码符号行：`commands.rs`、`lib.rs`、`index_host_app_entrypoints.rs`、`codex_db.rs`、`memory_context_entrypoints.rs`、`App.tsx`

边界：

- 未读取或写入 `/Users/yoyi/.codex`。
- 未读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- 未启动 GUI / Tauri dev，未运行 runner/test，未执行 `codex exec` / `codex exec resume`。
- 未写真实 workbench state root，未触碰真实产品数据，未改产品代码，未 `git add` / `git commit`。
- 本复核没有重新读取真实 workbench state root 内容；RU1 真实数据根清点事实以执行线 evidence 为证据，本线只核文档忠实性与源码阻断路径。

## Findings

- P0: none
- P1: none
- P2: none
- P3: none

## Key Verification

- RU1 分类忠实：evidence/handoff 均限定为 `partial_readonly_verified`，只声称真实 state root 与 `mariotest` 事实已只读核实；没有冒充 GUI 真机跑通、重开持久化已实测完成。
- RU2 分类忠实：evidence/handoff 均为 `blocked_not_executed`，明确未写 capture / observation / candidate / FormalMemory；没有冒充已写第一条真实正式记忆。
- RU3 分类忠实：evidence/handoff 均为 `blocked_deferred`，只给阻断分类与下一步选项；没有给出 B 可开、L5 完工线达成或产品已真用验证的正向结论。
- 默认 GUI / snapshot 阻断成立：`load_workbench_snapshot` 与 `query_workbench_page_read_model` 均构造 snapshot；`build_snapshot()` 固定传 `SessionSourceMode::RealWithSqliteFallback`；该路径进入 `load_sessions_from_sqlite_or_index` 并调用 `codex_db::default_state_db_path()`，后者从 `$HOME/.codex/state_*.sqlite` 或 `$HOME/.codex/state_5.sqlite` 定位并以 read-only sqlite 打开。该路径命中 RU 硬封印，因此不能在本窗口启动默认 GUI / Tauri dev 冒充 RU1 完成。
- RU2 安全非 GUI 入口阻断成立：现有 M2 采纳链路存在于 Tauri command / 前端 PermissionDialog pending action；后端 `adopt_memory_candidate_to_formal_memory_at` 先跑 lint guard 再写 FormalMemory。本轮未发现已存在的 RU 专用 CLI / runner / MCP 入口，能在不启动默认 GUI、不读 `.codex`、不改源码时完成 `capture -> observation -> candidate -> M2 adoption`。手工写 `formal-memories.v1.json` 或 `memory-candidates.v1.json` 会绕过 M2 门，evidence 对此分类准确。
- 边界陈述一致：新增 evidence/handoff 明确写明未读写 `.codex`、未执行真实 Codex、未写真实 workbench 数据根、未写 `mariotest` 项目、未改产品代码、未改 `CURRENT.md`。其中 evidence 也披露开工时已有 `CURRENT.md` 等咨询线未提交改动，未把它们归为本 RU 执行结果。

## Evidence Reviewed

本线只读执行了以下核验：

- `sed -n` 读取目标 evidence/handoff、kickoff、计划正本、`CURRENT.md` 首条。
- `rg -n` 定位 `load_workbench_snapshot`、`query_workbench_page_read_model`、`capture_memory_event`、`adopt_memory_candidate_to_formal_memory`、`adoptMemoryCandidateToFormalMemory` 等符号。
- `sed -n` 抽查 `commands.rs`、`lib.rs`、`index_host_app_entrypoints.rs`、`codex_db.rs`、`memory_context_entrypoints.rs`、`App.tsx` 的相关源码段。
- `git status --short -- ...` 仅检查目标 evidence/handoff/review 文件状态。

## Conclusion

本复核未发现阻断分类产物存在 overclaim。RU1 只能收为只读部分核实，RU2 必须保持 blocked_not_executed，RU3 必须保持 blocked_deferred；在当前硬封印下，不应启动默认 GUI、手写 JSON、或声称 B / L5 / 真实记忆写入已完成。建议提交前无需因本复核新增修复项。
