# R3 B0 Hash Calibration And Re-Freeze Handoff

日期：2026-06-15

状态：`completed`

## 结论

B0 / Level-B preflight 的 source aggregate hash 口径已统一到 canonical helper：

```text
workbench_source_aggregate_hash.v1:preflight_path_ref_file_hash_classification_concat
```

B0 re-freeze 后 canonical source root hash：

```text
31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
```

这不是源内容漂移：`workflow-state.v0.json` 与 `plan-authorizations.v1.json` 的单文件 hash 仍与 B0 原始清单一致。

## 代码变化

- `prototypes/productized-desktop-shell/src-tauri/src/utils/hash.rs`
  - 新增 `WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM`
  - 新增 `WorkbenchSourceAggregateHashEntry`
  - 新增 `workbench_source_aggregate_hash`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_preflight.rs`
  - preflight source root hash 改为调用 canonical helper
  - preflight report 新增 `source_root_hash_algorithm`
  - 新增普通测试证明 preflight report 与 canonical helper 一致
  - 新增 ignored readonly B0 calibration runner

## Evidence

- `evidence/r3-level-b/b0-hash-calibration-20260615-152136/README.md`
- `evidence/r3-level-b/b0-hash-calibration-20260615-152136/execution-record.json`
- `evidence/r3-level-b/b0-hash-calibration-20260615-152136/preflight-report.json`

## 边界

本包会写 repo evidence 下的 preflight report；除此之外保持 source-root readonly。未执行 B1 apply，未建 DB，未写 source root，未创建 backup / rollback manifest，未切读，未停写 JSON / sidecar，未执行真实 Codex，未读取或写入 `/Users/yoyi/.codex`。

## 验证与复核

- `cargo fmt -- --check`: pass
- `cargo test --lib sqlite_preflight -- --nocapture`: 9 passed / 1 ignored
- `cargo test --lib sqlite_production -- --nocapture`: 29 passed / 1 ignored
- `cargo test --lib`: 492 passed / 18 ignored
- `node scripts/harness/workbench-shape-gate.js --mode check`: pass, 0 errors / 0 warnings
- `git diff --check`: pass
- 复核线 Ampere (`019eca2b-6f24-7d11-bf3a-0dd74066ea85`): `CLEAR_WITH_P2`，P2 已修或澄清

## 下一步

提交后停在 B1 retry 前。B1 retry 需要用户在场重新确认 canonical expected hash 与输出路径。
