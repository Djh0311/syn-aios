# R3 Level B B1 Production Apply Failed Classified Handoff

日期：2026-06-15

状态：`failed_classified`

## 结论

B1 production apply 未完成。用户确认路径后，主管线用 B1 ignored runner 启动真实 apply，但在建 DB 前被 `source_root_hash_mismatch` 阻断：

```text
expected=2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53
actual=31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
```

本次不能声明 B1 完成、production DB 已建、backup / rollback manifest 已生成、read-cut 或 stop-write 已执行。

## 证据

- execution record: `evidence/r3-level-b/b1-production-apply-20260615-150005/execution-record.json`
- report: `evidence/r3-level-b/b1-production-apply-20260615-150005/README.md`
- 复核线：Jason (`019eca11-45c0-7ef2-bebb-ab64e77358b6`) 静态复核 `STATUS: CLEAR`

## 实际影响

- `r3-migration-work` 目录不存在。
- production DB 未创建。
- backup root 未创建。
- rollback manifest 未创建。
- production apply report 未创建。
- source root 仍只有 `workflow-state.v0.json` 与 `plan-authorizations.v1.json`。
- 两个源文件 SHA-256 与 B0 清单一致。

## 已通过验证

- `cargo test --lib sqlite_apply -- --nocapture`: 6 passed
- `cargo test --lib sqlite_export -- --nocapture`: 3 passed
- `cargo test --lib sqlite_production -- --nocapture`: 29 passed / 1 ignored
- `cargo fmt -- --check`: pass
- `node scripts/harness/workbench-shape-gate.js --mode check`: pass, 0 errors / 0 warnings
- `git diff --check`: pass
- `cargo test --lib`: 490 passed / 17 ignored

## 边界

本次未执行真实 Codex，未读取或写入 `/Users/yoyi/.codex`，未触碰 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout/prompt body，未改 Tauri/UI/app startup/product global read/write path，未执行 read-cut 或 stop-write。

## 下一步

先做 B0 hash-algorithm calibration / B0-refresh。不要直接用 `31cdea...` 重跑 B1。若校准确认源文件内容未变且当前 Level-B preflight hash 是正确 expected hash，再由用户确认 B1 retry。
