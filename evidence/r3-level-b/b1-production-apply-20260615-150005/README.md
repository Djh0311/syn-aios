# R3 Level B B1 Production Apply Attempt

日期：2026-06-15

状态：`failed_classified`

本窗口按用户确认路径启动 B1 production apply，但未完成。Level-B runner 在建 DB / 备份 / report / rollback manifest 之前命中 `source_root_hash_mismatch`，因此按硬中止条件停止。

## 用户确认路径

- `WORKBENCH_STATE_ROOT`: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`
- `PRODUCTION_DB_PATH`: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/workbench-state.v1.sqlite`
- `BACKUP_ROOT`: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/backup`
- `REPORT_PATH`: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/reports/production-apply-report.json`
- `ROLLBACK_MANIFEST_PATH`: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/rollback/rollback-manifest.json`

## Pre-Apply 验证

已通过：

```text
cargo test --lib sqlite_apply -- --nocapture
test result: ok. 6 passed; 0 failed; 0 ignored; 501 filtered out

cargo test --lib sqlite_export -- --nocapture
test result: ok. 3 passed; 0 failed; 0 ignored; 504 filtered out

cargo test --lib sqlite_production -- --nocapture
test result: ok. 29 passed; 0 failed; 1 ignored; 477 filtered out

cargo fmt -- --check
empty output

node scripts/harness/workbench-shape-gate.js --mode check
Status: pass
Errors: 0
Warnings: 0
Git HEAD: 370acd3b1eb004ce589881570ba54f794fbf4dea

git diff --check
empty output

cargo test --lib
test result: ok. 490 passed; 0 failed; 17 ignored
```

## Apply Attempt 结果

真实 B1 runner 命令用显式 env 授权启动，结果失败：

```text
running 1 test

thread 'workbench_sqlite_production_apply::tests::r3_b1_production_apply_confirmed_paths_requires_env_authorization' (...) panicked at src/workbench_sqlite_production_apply.rs:1956:10:
R3 B1 real production apply must complete: "production_apply_blocked:source_root_hash_mismatch:expected=2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53:actual=31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 506 filtered out
```

## 只读复查

失败后复查：

```text
find "/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state" -maxdepth 1 -type f -print
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/plan-authorizations.v1.json
```

```text
shasum -a 256 ".../workflow-state.v0.json" ".../plan-authorizations.v1.json"
4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972  workflow-state.v0.json
6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e  plan-authorizations.v1.json
```

这两个文件 hash 与 B0 evidence 一致。`r3-migration-work` 目录不存在：

```text
find: /Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work: No such file or directory
```

## 分类判断

本次失败分类为：`blocked_before_db_create_due_source_root_hash_mismatch`。

证据支持当前不是源文件内容变化：

- 源目录仍只有 B0 记录的两个文件。
- 两个源文件 SHA-256 与 B0 清单一致。
- 目标 migration work 目录未创建，说明未建 DB、未写 backup、未写 report、未写 rollback manifest。
- 当前代码的 preflight source root hash 函数使用 `path_ref + file_hash + classification`；B0 README 记录的算法是 `<path_sha256> <file_sha256> <relative_path>`。因此更可能是 B0 hash 口径和 Level-B preflight hash 口径不一致。

## 独立复核

复核线：Jason (`019eca11-45c0-7ef2-bebb-ab64e77358b6`)

结论：`STATUS: CLEAR`

复核范围：B1 runner 和 source backup diff 的静态复核。复核确认 Level-A 未放宽、Level-B 仍走 confirmed-path、source 只读并有 before / after invariant、未接 Tauri/UI/startup/read-cut/stop-write。

## 未做

- 未创建 production DB。
- 未创建 backup / rollback manifest / production apply report。
- 未切 read path。
- 未停写 JSON / sidecar。
- 未写 source JSON / sidecar。
- 未执行真实 Codex。
- 未读取或写入 `/Users/yoyi/.codex`。

## 下一步建议

不要直接把 `31cdea...` 替换进 B1 重跑。先开一个很小的 B0 hash-algorithm calibration / B0-refresh，只读确认：

- B0 记录的 aggregate hash 算法是否与当前 `scan_workbench_state_root_preflight_with_config` 不一致。
- 若源文件内容仍未变，应冻结“当前 Level-B preflight hash”作为 B1 retry 的 expected hash。
- 之后再由用户确认 B1 retry。
