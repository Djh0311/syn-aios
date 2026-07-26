# 知识库生产写路构建阻断修复：验收证据 v1

日期：2026-07-21  
任务包：`tasks/2026-07-21-knowledge-vault-production-write-path-build-blocker-package-v1.md`  
基线：`e9ad7f3`；执行线未 stage、未 commit

## 结论

知识库 audit 已不再调用仅在 `#[cfg(test)]` 下存在的
`workflow_state_store::atomic_write`。它固定通过既有 Batch 2 生产写路：

```rust
crate::write_m5b_batch2_workflow_state(
    workflow_state_path,
    "knowledge_vault_audit",
    &value,
)?;
```

non-test 编译与 Tauri debug build 均通过。JSON-only 测试使用合法
`workflow_state_v0` fixture；create -> create -> edit 的三个 audit 恰消费三个
revision。新增 DB-primary 测试实证 `source_kind=knowledge_vault` audit 在 DB 与
JSON 各一条、revision 只加一、重启 reconcile 为 green。

## 变更范围

- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs`
  - 替换 test-only raw writer；phase 固定为 `knowledge_vault_audit`。
  - 测试 fixture 补齐 validator 要求的 schema/version/revision/数组字段。
  - 锁三次审计后 `revision == 3`，AI 单次审计后 `revision == 1`。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs`
  - 新增窄 Batch 2 DB-primary audit 投影与 restart reconcile 测试。

未改 M5 bridge/repository/storage-mode 生产文件、schema/CAS、Blocked fallback、安全闸、命令注册、前端或现场数据。

## 验证

| 命令 / 核验 | 结果 |
|---|---|
| `cargo check --lib`（`src-tauri`） | exit 0；E0425 消失。仍有既有 unused/dead-code warnings，本包未清理。 |
| `cargo test --lib`（沙箱外复跑） | **1031 passed / 0 failed / 44 ignored**，exit 0。 |
| `cargo test --lib m5b` | **10 passed / 0 failed**，含新增知识库 audit case。 |
| `cargo test --lib m5c` | **5 passed / 0 failed**。 |
| `cargo test --lib m5f1` | **3 passed / 0 failed**；Blocked JSON-only 与陈旧 revision fail-closed 仍覆盖。 |
| `cargo-tauri build --debug`（只 build） | exit 0；未启动 App。 |
| `npm run typecheck` | exit 0。 |
| `npm run test:offline-interaction` | exit 0；15 组全绿。 |
| shape baseline | `13 errors / 5 warnings / 5 info`，同当前历史债务基线。 |
| shape check | exit 1，仍为相同的 13/5/5 历史债务（ratchet/unknown-sidecar 等不在本包修改面）；本包零新增。 |
| `git diff --check` | exit 0。 |

静态复核：`knowledge_vault.rs` 对 `workflow_state_store::atomic_write` 的引用为零；
`workflow_state_store.rs` 中该 helper 仍紧邻 `#[cfg(test)]`。

## 沙箱披露

首次沙箱内全量 `cargo test --lib` 为 `1029 passed / 2 failed / 44 ignored`。两个失败均在进程 mock fixture：

1. `codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child`：mock child PID 文件未出现。
2. `supervisor_session_launcher::resident_session_tests::s1b_h2_real_initial_and_resume_consume_only_the_private_submit_proposal_config`：读取 PID 的 `lstart` 被拒，报 `Operation not permitted`。

两条测试均使用临时 fake script，未启动真实 App；分别在沙箱外 exact rerun 通过，随后全量沙箱外复跑达到 1031/0/44。因此未改测试、进程/PID 逻辑或 H2。

## 范围与影响

`git diff --name-only` 的 tracked 修改仅为两份 Rust 文件；加上本 evidence 与对应 decision，最终写入面为合同允许的四个文件。工作树原有未跟踪 kickoff 与任务包保持不动。

- 新 Tauri command：0
- sidecar：0
- 依赖：0
- ratchet 文件：0
- 真实 App、workflow-state、DB、vault：0 次触碰

