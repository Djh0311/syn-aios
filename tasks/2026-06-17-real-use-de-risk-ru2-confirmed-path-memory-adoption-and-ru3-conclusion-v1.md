# RU2 Confirmed-Path Memory Adoption And RU3 Conclusion v1

日期：2026-06-17

状态：已完成，待咨询线/用户复核提交

## 拍板摘要

用户选择 RU 阻断记录中的方案一：补一个最窄的 confirmed-path RU/Dogfood 入口，在不读 `/Users/yoyi/.codex`、不启动 GUI、不跑真实 Codex 的前提下，使用真实 `mariotest` 项目把 `capture -> observation -> candidate -> M2 adoption -> FormalMemory` 跑通，并写 RU3 去险结论。代价是新增一个受测试保护的后端 runner 与真实 workbench state root 下的记忆 sidecar；不做则 RU2 继续阻断、L5 真记忆完工线无法兑现。

一句话判据：本包能否接受，只看真实正式记忆是否通过 confirmed-path runner 经 M2 adoption 写入真实 workbench state root，并且全过程没有读 `.codex`、没有启动 GUI、没有真实 Codex 执行、没有手写 JSON 冒充 M2。

## 目标

1. 新增 RU/Dogfood confirmed-path 后端入口，只允许调用方显式确认的 workflow state path / project root / project id / workflow id。
2. 入口走既有 `memory_capture_bus::capture_event` 与 M2 `adopt_memory_candidate_to_formal_memory_at`，不得手写 FormalMemory JSON。
3. 用真实 `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json` 写入一条来自 `mariotest` RU 真用判断的正式记忆。
4. 写 RU2/RU3 evidence 与 handoff，交独立复核线只读核。

## 允许写入

- `prototypes/productized-desktop-shell/src-tauri/src/ru_dogfood.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅注册 `mod ru_dogfood;`，如需调用 M2 helper 可做最小 visibility 调整）
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`（仅允许把既有 M2 helper 提升为 `pub(crate)`，不得改行为）
- 真实 workbench state root 下的记忆 sidecar：
  - `memory-capture-events.v1.json`
  - `observations.v1.json`
  - `memory-candidates.v1.json`
  - `formal-memories.v1.json`
  - `memory-lint.v1.json`
- `evidence/**`、`handoffs/**` 中本包记录与复核。

## 禁止

- 不读写 `/Users/yoyi/.codex`。
- 不读 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- 不启动 GUI / `tauri dev`。
- 不执行 `codex exec` / `codex exec resume`。
- 不启动 K3-B1/K3-B2。
- 不真 retry/stop/restart/resume。
- 不切 R3 产品全局 read/write path。
- 不手工写 FormalMemory JSON 绕过 M2。
- 不改前端 UI、不新增 Tauri command、不改 schema、不迁 SQLite。
- 不改咨询线已有 `CURRENT.md` / `AUTHORITY.md` dirty 内容。
- 不 `git add` / `git commit`。

## TDD 行为

先写失败测试，再实现：

1. confirmed path 不匹配时拒绝，不写 sidecar。
2. denied path（含 `.codex` / secret / token / `.env` / credential 等）拒绝。
3. fixture 上 successful path 必须生成 capture、observation、candidate、formal memory，并把 candidate adoption link 写回候选 store；正式记忆必须来自 M2 adoption warning / audit，而不是直接 create record。
4. ignored env runner 只有显式 `R3_RU2_DOGFOOD_CONFIRM=CONFIRMED_USER_PRESENT_2026_06_17` 时可跑；本包真实执行时使用该 runner。

## 真实 RU2 运行契约

在 `prototypes/productized-desktop-shell/src-tauri/` 下运行：

```text
R3_RU2_DOGFOOD_CONFIRM=CONFIRMED_USER_PRESENT_2026_06_17
R3_RU2_WORKFLOW_STATE_PATH="/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json"
R3_RU2_CONFIRMED_WORKFLOW_STATE_PATH="/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json"
R3_RU2_PROJECT_ROOT="/Users/yoyi/Documents/mario test"
R3_RU2_PROJECT_ID="project:users-yoyi-documents-mario-test"
R3_RU2_WORKFLOW_ID="workflow:users-yoyi-documents-mario-test:default"
cargo test --lib r3_ru2_dogfood_confirmed_paths_requires_env_authorization -- --ignored --nocapture
```

## 验证

- `cargo test --lib ru_dogfood`
- `cargo test --lib memory_capture_bus`
- `cargo test --lib memory_daily_loop`
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- `node scripts/harness/checkpoint-audit.js ...`

## 复核

独立复核线只读核：

- confirmed-path runner 是否没有 `.codex` 路径读取。
- 真实 FormalMemory 是否经 M2 adoption 写入。
- 真实 source/workflow state 主文件是否未改。
- 写入 sidecar 是否只在允许清单内。
- RU3 是否没有 overclaim：不开 B、不声称产品全局切库、不声称真实 Codex 已执行。

## 收口结果

- 代码入口：`prototypes/productized-desktop-shell/src-tauri/src/ru_dogfood.rs` 新增 test-only RU confirmed-path runner；`memory_context_entrypoints.rs` 仅做 test-only 注册与 M2 helper `pub(crate)` 可见性提升。
- 真实写入：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state` 下新增 5 个允许清单内记忆 sidecar；主 `workflow-state.v0.json` 与 `plan-authorizations.v1.json` hash 前后不变。
- 真记忆：`mem:v1:1781630651485:8a3140d6102a2c7d`，由 `capture -> observation -> candidate -> user confirm -> memory lint -> M2 adoption -> FormalMemory` 写入；不是 fixture，未手写 FormalMemory JSON。
- RU3 结论：L5 真记忆完工线窄口径达成；不建议立即开 B，建议先补真实 GUI/驾驶舱可视化与更自然的日常入口复核。
- 证据：`evidence/2026-06-17-real-use-de-risk-ru2-ru3-confirmed-path-memory-adoption-v1.md`。

## 不接受为

- 不接受为 GUI 真机跑通已验证。
- 不接受为 B 可开或 K3-B1/K3-B2 解锁。
- 不接受为产品全局 read/write path 已切 DB。
- 不接受为真实 Codex 执行。
- 不接受为所有记忆层真用都已充分验证。
