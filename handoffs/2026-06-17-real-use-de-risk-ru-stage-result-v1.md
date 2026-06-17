# 交接：真实使用去险小阶段（RU）结果 v1

日期：2026-06-17

出自：Codex 执行线。性质：RU2/RU3 confirmed-path 收口回交，供独立复核线、咨询线和用户拍板使用。

## 结果摘要

- RU1 状态：真实 `mario test` 项目和真实 workbench state root 已在前序阻断记录中只读核实；默认 GUI 路径因 `.codex` 读取风险未跑。
- RU2 状态：已通过窄 confirmed-path runner 真写入 1 条正式记忆，路径为真实 workbench state root，不是 fixture。
- RU3 状态：结论已写入 evidence；建议暂不立即开 B，先核实本包实物并补 GUI/驾驶舱真机复核。

## 关键实物

- 代码入口：`prototypes/productized-desktop-shell/src-tauri/src/ru_dogfood.rs`
- 注册/可见性：`prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- 任务包：`tasks/2026-06-17-real-use-de-risk-ru2-confirmed-path-memory-adoption-and-ru3-conclusion-v1.md`
- 证据：`evidence/2026-06-17-real-use-de-risk-ru2-ru3-confirmed-path-memory-adoption-v1.md`
- 结构化执行记录：`evidence/2026-06-17-real-use-de-risk-ru2-ru3-confirmed-path-memory-adoption-execution-record.json`
- 前序阻断记录：`evidence/2026-06-17-real-use-de-risk-ru1-ru2-blocked-v1.md`

## 真记忆

```text
state_root=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state
formal_sidecar=formal-memories.v1.json
formal_sidecar_sha256=3b2c13af745daf710fc3810a005d7af17ccc8e092ad416c6fc72aeae898becfe
memory_id=mem:v1:1781630651485:8a3140d6102a2c7d
candidate_key=memcand:v1:d52ec5fb5378ffb013219d0d6bd1e6f4b9682c195c436fe6ac0640dd88dae55d
observation_id=obs:v1:1781630651485:3313fafbc3954aa5
capture_event_id=memory-capture:1781630651485:a16cb8f0eef8
lint_status=succeeded
lint_blocking_count=0
```

## 边界

- 本次只新增 test-only RU confirmed-path runner；没有新增 Tauri command 或产品 UI。
- 主 `workflow-state.v0.json` 与 `plan-authorizations.v1.json` hash 前后不变。
- 写入只发生在真实 state root 的允许记忆 sidecar：`memory-capture-events.v1.json`、`observations.v1.json`、`memory-candidates.v1.json`、`formal-memories.v1.json`、`memory-lint.v1.json`。
- 未读写 `/Users/yoyi/.codex`，未执行真实 Codex，未切 DB，未停写 JSON/sidecar。

## 验证

- `cargo test --lib ru_dogfood -- --nocapture`：3 passed / 1 ignored。
- 真实 ignored runner：1 passed，输出 `status=completed`。
- `cargo test --lib memory_capture_bus`：8 passed。
- `cargo test --lib memory_daily_loop`：2 passed。
- `cargo test --lib memory_candidate`：9 passed。
- `cargo test --lib`：521 passed / 22 ignored。
- `cargo fmt -- --check`：exit 0。
- `rustfmt prototypes/productized-desktop-shell/src-tauri/src/ru_dogfood.rs`：exit 0。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 1 existing warning。
- `git diff --check`：exit 0。

## 给咨询线的核验建议

1. 只读打开真实 state root 的 5 个 sidecar，核对 hash 与 memory/adoption/lint 字段。
2. 核对 `workflow-state.v0.json` 与 `plan-authorizations.v1.json` 仍为 `4bd543...` / `6962e4...`。
3. 核对 `ru_dogfood.rs` 没有 `Command::new`、`codex exec`、`.codex` 读取路径或手写 FormalMemory JSON。
4. 核对 RU3 没有 overclaim：没有说 B 已开、GUI 已验、产品全局读写已切、真实 Codex 已执行。
