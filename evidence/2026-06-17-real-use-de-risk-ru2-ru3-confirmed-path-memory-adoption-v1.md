# RU2/RU3 Confirmed-Path Memory Adoption And Conclusion v1

日期：2026-06-17

阶段：Real-Use De-risk（RU，真实使用去险）

状态：`completed_pending_review`

## 拍板摘要

用户选择方案一后，本包补了一个最窄的 RU/Dogfood confirmed-path runner：不启动 GUI、不执行真实 Codex、不读写 `/Users/yoyi/.codex`，只允许显式确认的真实 `workflow-state.v0.json` + `mario test` 项目/workflow 进入 `capture -> observation -> candidate -> M2 adoption -> FormalMemory`。真实 RU2 已完成，第一条真实正式记忆写入工作台自有 state root；RU3 给出“暂不建议立即开 B”的证据化建议，最终是否开 B 仍由用户/咨询线裁决。

一句话判据：本包可接受为“L5 真记忆完工线窄口径达成”，不可接受为 GUI 真机体感已验、B 已解锁、产品全局读写切换、真实 Codex 已执行或记忆层全量真用验证完成。

## 代码入口

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/ru_dogfood.rs`（488 行）：test-only RU runner，包含 confirmed-path 校验、denied path guard、真实 workflow state 注册校验、capture 输入构造、用户确认 candidate、项目主管低风险 M2 adoption、fixture/负向/ignored real runner 测试。
- 修改 `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`：新增 `#[cfg(test)] #[path = "ru_dogfood.rs"] mod ru_dogfood;`；把既有 `adopt_memory_candidate_to_formal_memory_at` 从私有提升为 `pub(crate)`，行为未改。
- 未新增 Tauri command，未改前端 UI，未改 schema，未改 R3 DB/SQLite 切换逻辑，`lib.rs` shape 水线保持 `5567/5567`。

## 真实 RU2 写入

结构化执行记录：`evidence/2026-06-17-real-use-de-risk-ru2-ru3-confirmed-path-memory-adoption-execution-record.json`

真实 state root：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state
```

真实项目与 workflow：

```text
project_root=/Users/yoyi/Documents/mario test
project_id=project:users-yoyi-documents-mario-test
workflow_id=workflow:users-yoyi-documents-mario-test:default
workflow title=mario test 四角色编排测试工作流
nodes=7
edges=7
```

真实 runner 命令：

```text
R3_RU2_DOGFOOD_CONFIRM=CONFIRMED_USER_PRESENT_2026_06_17
R3_RU2_WORKFLOW_STATE_PATH="/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json"
R3_RU2_CONFIRMED_WORKFLOW_STATE_PATH="/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json"
R3_RU2_PROJECT_ROOT="/Users/yoyi/Documents/mario test"
R3_RU2_PROJECT_ID="project:users-yoyi-documents-mario-test"
R3_RU2_WORKFLOW_ID="workflow:users-yoyi-documents-mario-test:default"
cargo test --lib r3_ru2_dogfood_confirmed_paths_requires_env_authorization -- --ignored --nocapture
```

真实 runner 原始输出：

```text
running 1 test
{
  "candidate_key": "memcand:v1:d52ec5fb5378ffb013219d0d6bd1e6f4b9682c195c436fe6ac0640dd88dae55d",
  "candidate_store_revision": 3,
  "capture_event_id": "memory-capture:1781630651485:a16cb8f0eef8",
  "formal_memory_revision": 1,
  "memory_id": "mem:v1:1781630651485:8a3140d6102a2c7d",
  "observation_id": "obs:v1:1781630651485:3313fafbc3954aa5",
  "status": "completed",
  "warnings": [
    "formal_memory_store_m1_no_candidate_adoption_or_task_injection",
    "memory_candidate_adopted_to_formal_memory",
    "candidate_history_retained_with_adoption_link",
    "cross_sidecar_write_formal_then_candidate_link"
  ]
}
test ru_dogfood::tests::r3_ru2_dogfood_confirmed_paths_requires_env_authorization ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 542 filtered out; finished in 0.11s
```

主源文件 hash 前后不变：

```text
workflow-state.v0.json       4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972
plan-authorizations.v1.json  6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e
```

新增/更新的允许 sidecar hash：

```text
b771a51ec95e25fd830b33df23131d5ee45e7ffbd2fe9e6176caccf2a5d62191  memory-capture-events.v1.json
8c6258bc51829ab1ba6d233997402eee8dfba41628bb6831bee42d53f77718dc  observations.v1.json
3f0e850cd084a4ee019d9cc810fbf36be71eb5d29e6658bcd9f1c8c39c8dcdd9  memory-candidates.v1.json
3b2c13af745daf710fc3810a005d7af17ccc8e092ad416c6fc72aeae898becfe  formal-memories.v1.json
a68f6270c2d4836ee52c0037abe0317df22232448b780a3c41ca4f2d9ce83a58  memory-lint.v1.json
```

真实正式记忆：

```text
memory_id=mem:v1:1781630651485:8a3140d6102a2c7d
memory_type=workflow_summary
scope_type=workflow
project_id=project:users-yoyi-documents-mario-test
workflow_id=workflow:users-yoyi-documents-mario-test:default
claim=mario test RU 发现默认 GUI 真机路径会读 Codex state，记忆闭环需要 confirmed-path 入口。
status=memory_active
formal_store_revision=1
```

M2 链路核验：

- capture store：`revision=1`，1 event，`candidate_policy=candidate_allowed`，`created_by=user:ru-dogfood`。
- observation store：`revision=2`，1 observation，`status=candidate_created`，`risk_level=low`，`sensitive_level=internal`。
- candidate store：`revision=3`，1 candidate，`status=candidate_confirmed`，`requires_user_confirmation=false`，`sensitive_level=project`，有 user 确认 audit 与 adoption link。
- formal store：`revision=1`，1 record / 1 version / 1 audit event，audit type=`memory_candidate_adopted_to_formal_memory`。
- memory lint：`revision=1`，1 run，`lint_intent=candidate_adoption_guard`，`actor_role=project_director`，`blocking_count=0`，`status=succeeded`。

M2 角色边界说明：

- 候选确认由 `user:ru-dogfood` / `actor_role=user` 记录，表示用户在场确认候选可进入采纳。
- 正式采纳由 `project-director:ru-dogfood` / `actor_role=project_director` 执行，因为现有 `memory_lint` guard 只接受 `project_director | global_director | system`，且 M2 对 `project_director` 只允许低风险 project/workflow/session 记忆。
- 本包没有放宽 M2 或 lint；为符合既有规则，RU dogfood candidate 设为低风险 workflow summary，capture/observation 为 `internal`，映射到 candidate/formal source 的 `project` 级敏感度。

## RU3 结论

L5 完工线状态：

- 达成，窄口径：已在真实 workbench state root 写入 1 条真实正式记忆，非 fixture，带真实 `mario test` workflow 来源，并经 `memory_capture_bus` + M2 adoption + memory lint guard。
- 不扩大解释：这不等于所有记忆入口都真用验证，不等于前端日常收件箱视觉/交互已真机验收，不等于自动正式化。

可观察摩擦清单：

- 默认 GUI / snapshot 路径在当前安全封印下会牵出 `/Users/yoyi/.codex` 读取风险，因此 RU2 不能靠默认 GUI 硬跑；需要 confirmed-path runner 才能安全完成。
- M2 采纳的角色边界较隐蔽：候选用户确认与 lint/adoption actor 不是同一个角色，否则会被 `memory lint actor_role 不允许：user` 拦下；这对未来产品文案/开发者面板需要更清楚地解释。
- `cargo fmt` 没自动格式化这个 test-only path 模块，需要显式 `rustfmt`；这是工程接线摩擦，不影响产品数据。
- 我没有打开 GUI，因此只能报告这些可观察工程/流程摩擦，不能替用户判断驾驶舱“顺不顺手”。

是否开 B 的建议（非最终裁决）：

- 建议：暂不立即开 B。
- 理由：RU2 真记忆线已兑现，但默认 GUI 真机路径仍未在不读 `.codex` 的条件下完成；驾驶舱可视体感仍未由用户/GUI 验证；真实 Codex 执行封印还需要单独授权窗口与更强确认。
- 可接受的下一步：先让咨询线核实本 evidence 与真实 sidecar，再由用户决定是否开一个更窄的 GUI/驾驶舱真机复核或 B preflight；不要把本包当成 B 解锁依据。

## 验证原始输出

`cargo test --lib ru_dogfood -- --nocapture`

```text
running 4 tests
test ru_dogfood::tests::r3_ru2_dogfood_confirmed_paths_requires_env_authorization ... ignored, RU2 real dogfood runner requires explicit user-present env confirmation
test ru_dogfood::tests::ru_dogfood_rejects_denied_codex_path ... ok
test ru_dogfood::tests::ru_dogfood_rejects_unconfirmed_workflow_state_path ... ok
test ru_dogfood::tests::ru_dogfood_confirmed_fixture_writes_via_m2_adoption ... ok

test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 539 filtered out; finished in 0.11s
```

`cargo test --lib memory_capture_bus`

```text
running 8 tests
test memory_capture_bus::tests::memory_capture_rejects_secret_candidate_path ... ok
test memory_capture_bus::tests::memory_capture_corrupt_json_is_rejected_without_overwrite ... ok
test memory_capture_bus::tests::memory_capture_rejects_prompt_body_text ... ok
test memory_capture_bus::tests::memory_capture_audit_only_writes_no_observation_or_candidate ... ok
test memory_capture_bus::tests::memory_capture_revision_conflict_does_not_overwrite_store ... ok
test memory_capture_bus::tests::memory_capture_duplicate_event_is_rejected_without_append ... ok
test memory_capture_bus::tests::memory_capture_candidate_allowed_creates_observation_and_candidate_only ... ok
test memory_capture_bus::tests::operation_control_decision_can_be_captured_as_candidate_without_formal_memory ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 535 filtered out; finished in 0.05s
```

`cargo test --lib memory_daily_loop`

```text
running 2 tests
test memory_daily_loop::tests::l5_operation_control_capture_input_is_candidate_allowed_with_source_refs ... ok
test memory_daily_loop::tests::l5_daily_capture_creates_observation_and_candidate_without_formal_memory ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 541 filtered out; finished in 0.02s
```

`cargo test --lib memory_candidate`

```text
running 9 tests
test tests::memory_candidate_store_keeps_candidates_out_of_formal_memory ... ok
test tests::memory_candidate_rejection_does_not_create_formal_memory ... ok
test tests::memory_candidate_adoption_rejects_secret_without_blocked_export ... ok
test tests::memory_candidate_adoption_rejects_user_preference_without_user ... ok
test tests::memory_candidate_adoption_rejects_rejected_or_discarded_candidate ... ok
test tests::memory_candidate_adoption_rejects_context_binding_mismatch ... ok
test tests::memory_candidate_adoption_rejects_cross_project_project_director ... ok
test tests::memory_candidate_adoption_project_director_low_risk_project_memory ... ok
test tests::memory_candidate_adoption_rejects_already_adopted_candidate ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 534 filtered out; finished in 0.23s
```

`cargo test --lib`

```text
running 543 tests
...
test result: ok. 521 passed; 0 failed; 22 ignored; 0 measured; 0 filtered out; finished in 7.13s
```

`cargo fmt -- --check`

```text
<no output; exit 0>
```

`rustfmt prototypes/productized-desktop-shell/src-tauri/src/ru_dogfood.rs`

```text
<no output; exit 0>
```

`node scripts/harness/workbench-shape-gate.js --mode check`

```text
Status: pass
Errors: 0
Warnings: 1
Info: 9
Git HEAD: 512c04713f4a7403fb6b5fa7236ed59cfe0769b6
...
- lib.rs: 5567 lines (prototypes/productized-desktop-shell/src-tauri/src/lib.rs)
- Tauri commands: 98 total; 0 in lib.rs
...
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":98,"baseline":97}
```

`git diff --check`

```text
<no output; exit 0>
```

## 边界

- 未读写 `/Users/yoyi/.codex`，未读 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- 未执行 `codex exec` / `codex exec resume`，未启动 K3-B1/K3-B2，未真 retry/stop/restart/resume。
- 未启动 GUI / `tauri dev`，未改前端 UI，未新增 Tauri command。
- 未切 R3 产品全局 read/write path，未建/迁 SQLite，未停写 JSON/sidecar。
- 未手写 FormalMemory JSON；真实正式记忆由 M2 adoption 写入。
- 未改咨询线既有 `CURRENT.md` / `AUTHORITY.md` dirty 内容。

## 不接受为

- 不接受为 B 已解锁或真实 Codex 可开始。
- 不接受为 GUI 真机体感已验。
- 不接受为产品全局 read/write path 已切 DB。
- 不接受为完整存储迁移或 JSON stop-write。
- 不接受为记忆层全量真用验证完成。
