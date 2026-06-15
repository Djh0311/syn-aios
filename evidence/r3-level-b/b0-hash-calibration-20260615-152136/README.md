# R3 B0 Hash Calibration And Re-Freeze

日期：2026-06-15

状态：`completed`

本包统一 B0 / Level-B preflight 的 source aggregate hash 口径，并以 source-root readonly 方式 re-freeze B0 基线。本包会写 repo evidence 下的 preflight report；不执行 B1 apply，不创建 DB，不写 source root。

## 口径结论

旧 B0 evidence 记录的 aggregate hash 为：

```text
2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53
```

当前 canonical aggregate hash 为：

```text
31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
```

canonical algorithm id：

```text
workbench_source_aggregate_hash.v1:preflight_path_ref_file_hash_classification_concat
```

该口径由 `utils/hash.rs` 中的 `workbench_source_aggregate_hash` 提供，并由 `scan_workbench_state_root_preflight_with_config` 调用。

## 只读 Re-Freeze

执行命令：

```text
R3_B0_HASH_CALIBRATION_CONFIRM=CONFIRMED_READONLY_2026_06_15 ...
cargo test --lib r3_b0_hash_calibration_real_workbench_state_root_requires_env_authorization -- --ignored --nocapture
```

关键输出：

```text
R3_B0_CANONICAL_SOURCE_ROOT_HASH=31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
R3_B0_SOURCE_ROOT_HASH_ALGORITHM=workbench_source_aggregate_hash.v1:preflight_path_ref_file_hash_classification_concat
R3_B0_PREFLIGHT_REPORT_PATH=/Users/yoyi/workspace/product-line/evidence/r3-level-b/b0-hash-calibration-20260615-152136/preflight-report.json
test result: ok. 1 passed; 0 failed
```

preflight report hash：

```text
c96ccd90ad6837e6a8e434ceb74a8ad1202f6a32f6bdaad22ccb61b62f3c8dc4
```

## Source 文件确认

`WORKBENCH_STATE_ROOT` 下仍只有两个文件：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/plan-authorizations.v1.json
```

单文件 hash 与 B0 原清单一致：

```text
workflow-state.v0.json
4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972

plan-authorizations.v1.json
6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e
```

preflight summary：

```text
status=preflight_ready
files_accepted=2
files_missing_optional=13
files_rejected=0
blocked_reasons=0
```

## 分类

本次校准分类为：`aggregate_hash_algorithm_divergence_not_source_content_drift`。

含义：B1 首次尝试被挡住是正确的；但阻断来自旧 B0 evidence 的 aggregate hash 口径与真实 Level-B preflight 口径不一致，而不是源文件内容漂移。

## 未做

- 未执行 B1 production apply。
- 未创建 production DB。
- 未创建 backup / rollback manifest / production apply report。
- 未切 read path。
- 未停写 JSON / sidecar。
- 未写 `WORKBENCH_STATE_ROOT`。
- 未执行真实 Codex。
- 未读取或写入 `/Users/yoyi/.codex`。

## 验证与复核

验证通过：

```text
cargo fmt -- --check
cargo test --lib sqlite_preflight -- --nocapture
test result: ok. 9 passed; 0 failed; 1 ignored

cargo test --lib sqlite_production -- --nocapture
test result: ok. 29 passed; 0 failed; 1 ignored

cargo test --lib
test result: ok. 492 passed; 0 failed; 18 ignored

node scripts/harness/workbench-shape-gate.js --mode check
Status: pass
Errors: 0
Warnings: 0

git diff --check
empty output
```

独立复核线 Ampere (`019eca2b-6f24-7d11-bf3a-0dd74066ea85`) 结论为 `CLEAR_WITH_P2`。P2 已处理：runner 补了 `files_seen == 2`、`files_rejected == 0`、`blocked_reasons == 0` 断言；文档口径收紧为 source-root readonly。

## B1 Retry 前置

B1 retry 只能在用户重新确认后开始，并应使用 canonical expected source hash：

```text
31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
```
