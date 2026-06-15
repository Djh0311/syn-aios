# R3 B4a Level-B Stop-Write Decision Entry Evidence v1

日期：2026-06-16

状态：review_clear_pending_consultation_commit

## Scope

- 代码：`prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- 任务包：`tasks/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-v1.md`
- 复核记录：`evidence/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-review-parfit-v1.md`

本包新增 Level-B confirmed-path stop-write decision 入口；不执行 B4b，不跑真实 env runner，不触碰真实 state root / B1 DB，不执行真实 stop-write。

## Change Summary

- 新增 `SqliteStopWriteLevelBConfig` 和 `rehearse_stop_write_decision_level_b_workbench_owned_state`。
- Level-B 校验要求 DB、fallback root、projection root、observation report、work dir、stop-write report、rollback manifest 均与调用方 confirmed path 精确匹配。
- Level-B 只放开 confirmed DB 的 temp 限制；输出 report / rollback 必须在 confirmed work dir 内，且不得落入 source root、DB 目录或 projection root。
- 复用 confirmed DB export dry-run 读取路径；未新增任何实际停写 JSON、删除 sidecar、切产品全局读写路径、UI / Tauri / startup 接入。
- `approve_stop_write` 通过所有前置条件后的最好状态仍为 `ready_but_not_executed`；rollback 仍是 dry-run decision manifest。
- 新增 ignored runner `r3_b4_stop_write_decision_confirmed_paths_requires_env_authorization`，本包未运行。
- 新增负向与边界测试覆盖：非 confirmed DB、输出越界、prepare_only、evidence/hash precondition block、Level-A 非 temp DB 仍拒。

## Verification Output

`cargo test --lib sqlite_stop_write`

```text
running 23 tests
test workbench_sqlite_stop_write::tests::r3_b4_stop_write_decision_confirmed_paths_requires_env_authorization ... ignored, requires explicit R3 B4 stop-write decision authorization and confirmed paths
...
test result: ok. 22 passed; 0 failed; 1 ignored; 0 measured; 501 filtered out; finished in 1.04s
```

`cargo test --lib`

```text
running 524 tests
...
test result: ok. 503 passed; 0 failed; 21 ignored; 0 measured; 0 filtered out; finished in 10.40s
```

`cargo fmt -- --check`

```text
<no output; exit 0>
```

`node scripts/harness/workbench-shape-gate.js --mode check`

```text
Status: pass
Errors: 0
Warnings: 0
Git HEAD: 11beb3bf8bafaf78819b1d534deb185603168e89
```

`git diff --check`

```text
<no output; exit 0>
```

## Review

- Review line：Parfit
- Agent id：Parfit
- Review file：`evidence/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-review-parfit-v1.md`
- STATUS：CLEAR
- Findings：无 P0 / P1 / P2 / P3

Parfit 复核确认：Level-A 旧门未放宽；Level-B 是并列 confirmed-path 入口；未发现真实 stop-write JSON、删除 sidecar、产品全局读写路径、UI / Tauri / startup 接入；ignored runner 未运行。

## Boundaries

- 未运行 B4b。
- 未执行真实 stop-write。
- 未停写 JSON / sidecar。
- 未切产品全局读写路径。
- 未触碰真实 source root / B1 DB。
- 未触碰 `/Users/yoyi/.codex`。
- 未执行真实 Codex。
- 未提交，当前停在咨询线复扫 / commit 前。
