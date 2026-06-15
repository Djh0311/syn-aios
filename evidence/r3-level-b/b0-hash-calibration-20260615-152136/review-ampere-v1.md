# R3 B0 Hash Calibration Review - Ampere

日期：2026-06-15

复核线：Ampere (`019eca2b-6f24-7d11-bf3a-0dd74066ea85`)

状态：`CLEAR_WITH_P2`

## 结论

无 P0 / P1 blocker。

复核确认：

- `utils/hash.rs` 已抽出 canonical helper 和算法 ID。
- `workbench_sqlite_preflight.rs` 已直接调用同一 helper，不是只改 evidence 数字。
- preflight report 暴露 `source_root_hash_algorithm`。
- 普通测试覆盖同一组 files 经 helper 与 preflight report hash 一致。
- 未修改 Level-A / B1 confirmed-path guard。
- evidence / handoff 未声明 B1 完成，只声明 B1 retry 前置。

## P2 与处理

- P2：ignored B0 runner 不是完全不写文件，会写 repo evidence 下的 preflight report；措辞应为 `source-root readonly`。
  - 处理：README / handoff / checkpoint 统一使用 source-root readonly / 不写 source root 口径。
- P2：runner 可显式断言 `files_seen == 2` 和 `files_rejected == 0`。
  - 处理：已补 `files_seen == 2`、`files_accepted == 2`、`files_rejected == 0`、`blocked_reasons == 0` 断言，并重跑聚焦测试与真实 B0 calibration runner 通过。

## 复核边界

复核线只做只读审查，未运行真实 apply，未触碰 `/Users/yoyi/.codex`。
